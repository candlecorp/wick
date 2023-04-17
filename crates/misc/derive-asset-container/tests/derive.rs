use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::Result;
use asset_container::{Asset, AssetManager, Progress, Status};
use bytes::BytesMut;
use futures::StreamExt;
use tokio::io::AsyncReadExt;

#[derive(derive_asset_container::AssetManager)]
#[asset(TestAsset)]
struct Struct {
  field: TestAsset,
  inner: InnerStruct,
}

#[derive(derive_asset_container::AssetManager)]
#[asset(TestAsset)]
struct InnerStruct {
  field: TestAsset,
}

#[derive(derive_asset_container::AssetManager)]
#[asset(TestAsset)]
struct Struct2 {
  one: TestAsset,
  #[asset(skip)]
  #[allow(unused)]
  two: TestAsset,
  #[asset(skip)]
  #[allow(unused)]
  three: String,
}

#[tokio::test]
async fn test_skip() -> Result<()> {
  let s = Struct2 {
    one: TestAsset::new("Cargo.toml"),
    two: TestAsset::new("Cargo.toml"),
    three: "hello".to_owned(),
  };
  let assets = s.assets();
  assert_eq!(assets.len(), 1);
  Ok(())
}

#[tokio::test]
async fn test_progress() -> Result<()> {
  let s = Struct {
    field: TestAsset::new("Cargo.toml"),
    inner: InnerStruct {
      field: TestAsset::new("Cargo.toml"),
    },
  };
  let assets = s.assets();
  assert_eq!(assets.len(), 2);
  let mut progress = assets.pull_with_progress(());
  let mut asset_done = 0;
  let mut num_progress = 0;
  while let Some(progress) = progress.next().await {
    println!("Progress {:?}", progress);
    num_progress += 1;
    assert_eq!(progress.len(), 2);
    for progress in progress {
      match progress.status {
        Status::AssetComplete(_) => {
          asset_done += 1;
        }
        Status::PullFinished => {}
        Status::Progress { .. } => {}
        Status::Error(e) => {
          panic!("error:{}", e);
        }
      }
    }
  }
  assert_eq!(asset_done, 2);
  assert!(num_progress > 0);
  Ok(())
}

#[tokio::test]
async fn test_enums() -> Result<()> {
  let s = TestEnum::One(TestAsset::new("Cargo.toml"));
  let assets = s.assets();
  assert_eq!(assets.len(), 1);
  let mut progress = assets.pull_with_progress(());
  let mut num_progress = 0;
  let mut asset_done = 0;
  while let Some(progress) = progress.next().await {
    println!("Progress {:?}", progress);
    num_progress += 1;
    assert_eq!(progress.len(), 1);
    for progress in progress {
      match progress.status {
        Status::AssetComplete(_) => {
          asset_done += 1;
        }
        Status::PullFinished => {}
        Status::Progress { .. } => {}
        Status::Error(e) => {
          panic!("error:{}", e);
        }
      }
    }
  }
  assert_eq!(asset_done, 1);
  assert!(num_progress > 0);
  Ok(())
}

#[derive(derive_asset_container::AssetManager)]
#[asset(TestAsset)]
enum TestEnum {
  One(TestAsset),
  Two(Struct),
  Three(InnerStruct),
}

#[tokio::test]
async fn test_enums_2() -> Result<()> {
  let s = TestEnum::Two(Struct {
    field: TestAsset::new("Cargo.toml"),
    inner: InnerStruct {
      field: TestAsset::new("Cargo.toml"),
    },
  });
  let assets = s.assets();
  assert_eq!(assets.len(), 2);
  let s = TestEnum::Three(InnerStruct {
    field: TestAsset::new("Cargo.toml"),
  });
  let assets = s.assets();
  assert_eq!(assets.len(), 1);

  Ok(())
}

struct TestAsset {
  path: PathBuf,
}

impl TestAsset {
  fn new(path: impl AsRef<Path>) -> Self {
    Self {
      path: path.as_ref().to_path_buf(),
    }
  }
}

impl Asset for TestAsset {
  type Options = ();

  fn update_baseurl(&self, _baseurl: &str) {
    unimplemented!()
  }

  fn fetch_with_progress(
    &self,
    _options: Self::Options,
  ) -> std::pin::Pin<Box<dyn futures::Stream<Item = asset_container::Progress> + Send + '_>> {
    let mut file = match std::fs::File::open(&self.path) {
      Ok(file) => file,
      Err(err) => {
        return Box::pin(futures::stream::once(async move {
          Progress::new(self.name(), Status::Error(err.to_string()))
        }));
      }
    };

    let file_size = match file.metadata() {
      Ok(metadata) => metadata.len(),
      Err(err) => {
        return Box::pin(futures::stream::once(async move {
          Progress::new(self.name(), Status::Error(err.to_string()))
        }));
      }
    };

    let mut bytes_read = 0;
    let mut buffer = [0u8; 1024];
    let mut done = false;
    let mut rv = BytesMut::new();
    Box::pin(futures::stream::poll_fn(move |cx| {
      if done {
        return std::task::Poll::Ready(None);
      }
      let mut bytes_buffered = 0;
      loop {
        let bytes = match file.read(&mut buffer) {
          Ok(0) => break,
          Ok(bytes) => bytes,
          Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
          Err(e) => {
            return std::task::Poll::Ready(Some(Progress::new(self.name(), Status::Error(e.to_string()))));
          }
        };
        bytes_read += bytes as u64;
        rv.extend_from_slice(&buffer[0..bytes]);
        bytes_buffered += bytes as u32;

        if bytes_read >= file_size {
          break;
        }

        if bytes_buffered >= 1024 {
          break;
        }
      }

      if bytes_buffered > 0 {
        std::task::Poll::Ready(Some(Progress::new(
          self.name(),
          Status::Progress {
            num: bytes_read as _,
            total: file_size as _,
          },
        )))
      } else if bytes_read >= file_size {
        done = true;
        std::task::Poll::Ready(Some(Progress::new(self.name(), Status::AssetComplete(rv.to_vec()))))
      } else {
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
      }
    }))
  }

  fn fetch(
    &self,
    _options: Self::Options,
  ) -> std::pin::Pin<
    Box<dyn futures::Future<Output = std::result::Result<Vec<u8>, asset_container::Error>> + Send + Sync>,
  > {
    let path = self.path.clone();
    Box::pin(async move {
      let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|err| asset_container::Error::FileOpen(path.to_string_lossy().to_string(), err.to_string()))?;
      let mut bytes = Vec::new();
      file.read_to_end(&mut bytes).await?;
      Ok(bytes)
    })
  }

  fn name(&self) -> &str {
    self.path.to_str().unwrap()
  }
}
