use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use asset_container::{Asset, AssetManager, Assets, Progress, Status};
use bytes::BytesMut;
use futures::StreamExt;
use tokio::io::AsyncReadExt;

struct TestConfig {
  this: LocationReference,
  that: LocationReference,
}

#[derive(Clone)]
struct LocationReference {
  path: PathBuf,
  baseurl: Arc<Mutex<Option<String>>>,
}

impl LocationReference {
  fn new(path: impl AsRef<Path>) -> Self {
    Self {
      path: path.as_ref().to_path_buf(),
      baseurl: Default::default(),
    }
  }
}

impl Asset for LocationReference {
  type Options = ();

  fn update_baseurl(&self, baseurl: &str) {
    self.baseurl.lock().unwrap().replace(baseurl.to_owned());
  }

  fn fetch_with_progress(
    &self,
    _options: Self::Options,
  ) -> std::pin::Pin<Box<dyn futures::Stream<Item = Progress> + Send + '_>> {
    let mut path = self
      .baseurl
      .lock()
      .unwrap()
      .as_ref()
      .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);
    path.push(&self.path);
    let mut file = match std::fs::File::open(&path) {
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

  fn name(&self) -> &str {
    self.path.to_str().unwrap()
  }

  fn fetch(
    &self,
    _options: Self::Options,
  ) -> std::pin::Pin<
    Box<dyn futures::Future<Output = std::result::Result<Vec<u8>, asset_container::Error>> + Send + Sync>,
  > {
    let mut path = self
      .baseurl
      .lock()
      .unwrap()
      .as_ref()
      .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);
    path.push(&self.path);
    Box::pin(async move {
      let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|err| asset_container::Error::FileOpen(path.to_string_lossy().to_string(), err.to_string()))?;
      let mut bytes = Vec::new();
      file.read_to_end(&mut bytes).await?;
      Ok(bytes)
    })
  }
}

impl AssetManager for TestConfig {
  type Asset = LocationReference;

  fn set_baseurl(&self, _baseurl: &str) {}

  fn assets(&self) -> Assets<Self::Asset> {
    let mut assets = Assets::default();
    assets.push(&self.this);
    assets.push(&self.that);
    assets
  }
}

#[test_logger::test(tokio::test)]
async fn test_basics() -> Result<()> {
  let config = TestConfig {
    this: LocationReference::new("../Cargo.toml"),
    that: LocationReference::new("../README.md"),
  };
  let mut assets = config.assets();
  assets.set_baseurl("tests");
  assert_eq!(assets.len(), 2);
  let mut progress = assets.pull_with_progress(());
  let mut num_progress = 0;
  let mut asset_done = 0;
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
  assert!(num_progress > 0);
  assert_eq!(asset_done, 2);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_extend() -> Result<()> {
  let config = TestConfig {
    this: LocationReference::new("Cargo.toml"),
    that: LocationReference::new("README.md"),
  };
  let config2 = TestConfig {
    this: LocationReference::new("Cargo.toml"),
    that: LocationReference::new("README.md"),
  };
  let mut assets = config.assets();
  assets.extend(&mut config2.assets());
  assert_eq!(assets.len(), 4);

  Ok(())
}
