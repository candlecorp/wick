use std::borrow::Cow;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::task::Poll;

use futures::{Future, FutureExt, Stream, StreamExt};

use crate::Error;

#[derive(Debug)]
#[must_use]
pub struct Assets<'a, T>
where
  T: Asset + Clone,
{
  list: Vec<Cow<'a, T>>,
}

impl<'a, T> Default for Assets<'a, T>
where
  T: Asset + Clone,
{
  fn default() -> Self {
    Self { list: Vec::new() }
  }
}

impl<'a, T> Assets<'a, T>
where
  T: Asset + Clone,
{
  pub fn new(list: Vec<Cow<'a, T>>) -> Self {
    Self { list }
  }

  #[must_use]
  pub fn list(&self) -> &[Cow<'a, T>] {
    &self.list
  }

  #[must_use]
  pub fn to_owned(self) -> Vec<T> {
    self.list.into_iter().map(|asset| asset.into_owned()).collect()
  }

  pub fn set_baseurl(&self, baseurl: &str) {
    self.list.iter().for_each(|asset| asset.update_baseurl(baseurl));
  }

  pub fn iter(&mut self) -> impl Iterator<Item = &Cow<'a, T>> {
    self.list.iter()
  }

  #[must_use]
  pub fn len(&self) -> usize {
    self.list.len()
  }

  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.list.is_empty()
  }

  pub fn push(&mut self, asset: &'a T) {
    self.list.push(Cow::Borrowed(asset));
  }

  pub fn push_owned(&mut self, asset: T) {
    self.list.push(Cow::Owned(asset));
  }

  #[allow(clippy::needless_pass_by_value)]
  pub fn pull(&'a mut self, options: T::Options) -> AssetPull<'a> {
    AssetPull::new(self, &options)
  }

  #[allow(clippy::needless_pass_by_value)]
  pub fn pull_with_progress(&'a mut self, options: T::Options) -> AssetPullWithProgress<'a> {
    AssetPullWithProgress::new(self, &options)
  }

  pub fn extend(&mut self, other: &mut Self) {
    self.list.append(&mut other.list);
  }
}

#[must_use]
pub struct AssetProgress<'a> {
  name: String,
  progress: Pin<Box<dyn Stream<Item = Progress> + Send + 'a>>,
}

impl std::fmt::Debug for AssetProgress<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AssetProgress").field("name", &self.name).finish()
  }
}

#[must_use]
pub struct AssetFut<'a> {
  name: String,
  progress: Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send + 'a>>,
  finished: AtomicBool,
}

impl std::fmt::Debug for AssetFut<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AssetFut").field("name", &self.name).finish()
  }
}

#[must_use]
pub struct CompleteAsset {
  name: String,
  result: Result<Vec<u8>, Error>,
}

impl std::fmt::Debug for CompleteAsset {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CompleteAsset").field("name", &self.name).finish()
  }
}

impl CompleteAsset {
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn result(&self) -> &Result<Vec<u8>, Error> {
    &self.result
  }

  pub fn into_bytes(self) -> Result<Vec<u8>, Error> {
    self.result
  }
}

#[must_use]
#[allow(missing_debug_implementations)]
pub struct AssetPull<'a> {
  assets: Vec<AssetFut<'a>>,
  finished: AtomicBool,
}

impl<'a> AssetPull<'a> {
  pub fn new<T>(assets: &'a mut Assets<'a, T>, options: &T::Options) -> Self
  where
    T: Asset + Clone,
  {
    let assets = assets
      .iter()
      .map(|asset| AssetFut {
        name: asset.name().to_owned(),
        progress: asset.fetch(options.clone()),
        finished: AtomicBool::new(false),
      })
      .collect();
    Self {
      assets,
      finished: AtomicBool::new(false),
    }
  }
}

impl<'a> Future for AssetPull<'a> {
  type Output = Result<Vec<CompleteAsset>, Error>;

  fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
    let mut all_done = true;
    let mut results = Vec::new();
    let this = self.get_mut();
    for asset in this.assets.iter_mut() {
      let name = &asset.name;
      let fut = &mut asset.progress;
      if asset.finished.load(std::sync::atomic::Ordering::Relaxed) {
        continue;
      }
      match fut.poll_unpin(cx) {
        Poll::Ready(result) => {
          asset.finished.store(true, std::sync::atomic::Ordering::Relaxed);
          results.push(CompleteAsset {
            name: name.clone(),
            result,
          });
        }
        Poll::Pending => {
          all_done = false;
        }
      }
    }
    if this.finished.load(std::sync::atomic::Ordering::Relaxed) {
      Poll::Ready(Ok(results))
    } else if all_done {
      this.finished.store(true, std::sync::atomic::Ordering::Relaxed);
      std::task::Poll::Ready(Ok(results))
    } else {
      std::task::Poll::Pending
    }
  }
}

#[must_use]
#[allow(missing_debug_implementations)]
pub struct AssetPullWithProgress<'a> {
  assets: Vec<AssetProgress<'a>>,
  finished: AtomicBool,
}

impl<'a> AssetPullWithProgress<'a> {
  pub fn new<T>(assets: &'a mut Assets<'a, T>, options: &T::Options) -> Self
  where
    T: Asset + Clone,
  {
    let assets = assets
      .iter()
      .map(|asset| AssetProgress {
        name: asset.name().to_owned(),
        progress: asset.fetch_with_progress(options.clone()),
      })
      .collect();
    Self {
      assets,
      finished: AtomicBool::new(false),
    }
  }
}

impl<'a> Stream for AssetPullWithProgress<'a> {
  type Item = Vec<Progress>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
    let mut all_done = true;
    let mut progress = Vec::new();
    for asset in self.assets.iter_mut() {
      let name = &asset.name;
      let stream = &mut asset.progress;
      match stream.poll_next_unpin(cx) {
        Poll::Ready(Some(p)) => {
          if !matches!(
            p.status,
            Status::AssetComplete(_) | Status::PullFinished | Status::Error(_)
          ) {
            all_done = false;
          }
          progress.push(p);
        }
        Poll::Ready(None) => {
          progress.push(Progress {
            status: Status::PullFinished,
            asset: name.clone(),
          });
        }
        Poll::Pending => {
          all_done = false;
        }
      }
    }
    if self.finished.load(std::sync::atomic::Ordering::Relaxed) {
      Poll::Ready(None)
    } else if all_done {
      self.finished.store(true, std::sync::atomic::Ordering::Relaxed);
      std::task::Poll::Ready(Some(progress))
    } else if progress.is_empty() {
      cx.waker().wake_by_ref();
      std::task::Poll::Pending
    } else {
      std::task::Poll::Ready(Some(progress))
    }
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Progress {
  pub status: Status,
  pub asset: String,
}

impl Progress {
  pub fn new(asset: impl AsRef<str>, status: Status) -> Self {
    Self {
      status,
      asset: asset.as_ref().to_owned(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Status {
  Error(String),
  AssetComplete(Vec<u8>),
  PullFinished,
  Progress { num: usize, total: usize },
}

pub trait AssetManager {
  type Asset: Asset + Clone;
  fn assets(&self) -> Assets<Self::Asset>;
  fn set_baseurl(&self, baseurl: &str);
}

pub trait Asset {
  type Options: Clone;
  fn fetch_with_progress(&self, options: Self::Options) -> Pin<Box<dyn Stream<Item = Progress> + Send + '_>>;
  fn fetch(&self, options: Self::Options) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send + Sync + '_>>;
  fn name(&self) -> &str;
  fn update_baseurl(&self, baseurl: &str);
}

impl<T, O> Asset for Option<T>
where
  T: Asset<Options = O> + Send + Sync + 'static,
  O: Clone + Send + Sync + 'static,
{
  type Options = T::Options;

  fn fetch_with_progress(&self, options: Self::Options) -> Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    self.as_ref().map_or_else(
      || {
        let stream = futures::stream::once(async move { Progress::new("None", Status::Error("None".to_owned())) });
        let a = stream.boxed();
        a
      },
      |a| {
        let a = a.fetch_with_progress(options);
        a
      },
    )
  }

  fn fetch(&self, options: Self::Options) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send + Sync + '_>> {
    Box::pin(async move {
      match self {
        Some(a) => a.fetch(options).await,
        None => Err(Error::FileNotFound("None".to_owned())),
      }
    })
  }

  fn name(&self) -> &str {
    self.as_ref().map_or("", |a| a.name())
  }

  fn update_baseurl(&self, baseurl: &str) {
    if let Some(asset) = self.as_ref() {
      asset.update_baseurl(baseurl);
    }
  }
}

impl<T> AssetManager for Option<T>
where
  T: AssetManager,
{
  type Asset = T::Asset;

  fn set_baseurl(&self, baseurl: &str) {
    if let Some(asset) = self.as_ref() {
      asset.set_baseurl(baseurl);
    }
  }

  fn assets(&self) -> Assets<Self::Asset> {
    self.as_ref().map(|a| a.assets()).unwrap_or_default()
  }
}

impl<K, T> AssetManager for std::collections::HashMap<K, T>
where
  T: AssetManager,
{
  type Asset = T::Asset;

  fn set_baseurl(&self, baseurl: &str) {
    for asset in self.values() {
      asset.set_baseurl(baseurl);
    }
  }

  fn assets(&self) -> Assets<Self::Asset> {
    let mut assets = Assets::default();
    for (_, asset) in self.iter() {
      assets.extend(&mut asset.assets());
    }
    assets
  }
}

impl<T> AssetManager for Vec<T>
where
  T: AssetManager,
{
  type Asset = T::Asset;

  fn set_baseurl(&self, baseurl: &str) {
    for asset in self.iter() {
      asset.set_baseurl(baseurl);
    }
  }

  fn assets(&self) -> Assets<Self::Asset> {
    let mut assets = Assets::default();
    for asset in self.iter() {
      assets.extend(&mut asset.assets());
    }
    assets
  }
}
