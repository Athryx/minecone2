use std::path::{PathBuf, Path};
use std::fs;
use std::sync::LazyLock;
use rustc_hash::FxHashMap;
use parking_lot::RwLock;
use std::sync::Arc;

use anyhow::Result;
use image::DynamicImage;

use crate::render::RenderContext;
use crate::render::model::Model;

static LOADER: LazyLock<AssetLoader> = LazyLock::new(|| AssetLoader::from_path(PathBuf::from("res/")));

pub fn loader() -> &'static AssetLoader {
	&LOADER
}

// this is realy basic for now, may be improved in future
pub struct AssetLoader {
	resource_folder: PathBuf,
	cached_models: RwLock<FxHashMap<PathBuf, Arc<Model>>>,
}

impl AssetLoader {
	fn from_path(resource_folder: PathBuf) -> Self {
		Self {
			resource_folder,
			cached_models: RwLock::new(FxHashMap::default()),
		}
	}

	fn path_of<T: AsRef<Path>>(&self, resource: T) -> PathBuf {
		let mut path = self.resource_folder.clone();
		path.push(resource);
		path
	}

	pub fn load_bytes<T: AsRef<Path>>(&self, file: T) -> Result<Vec<u8>> {
		Ok(fs::read(&self.path_of(file))?)
	}

	pub fn load_image<T: AsRef<Path>>(&self, file: T) -> Result<DynamicImage> {
		Ok(image::open(&self.path_of(file))?)
	}

	/*pub fn load_obj<T: AsRef<Path>>(&self, file: T) -> Result<(Vec<tobj::Model>, Vec<tobj::Material>)> {
		let (obj_meshes, obj_materials) = tobj::load_obj(&self.path_of(file), &tobj::GPU_LOAD_OPTIONS)?;
		let obj_materials = obj_materials?;
		Ok((obj_meshes, obj_materials))
	}*/

	/*pub fn load_model_cached<T: AsRef<Path>>(&self, file: T, context: RenderContext) -> Result<Arc<Model>> {
		let file = file.as_ref();

		// don't use an upgradeable read lock guard,
		// because that requires all other upgradable locks to be finished,
		// and most of the time the code path without the write part will be taken
		let read_lock = self.cached_models.read();

		if let Some(model) = read_lock.get(file) {
			Ok(model.clone())
		} else {
			drop(read_lock);

			// we will probably need to insert a PathBuf after this, so do the expensive allocation while no lock is held
			let file_owned = file.to_owned();
			let path = self.path_of(file);

			let mut write_lock = self.cached_models.write();

			if let Some(model) = write_lock.get(file) {
				Ok(model.clone())
			} else {
				let model = Arc::new(Model::load_from_file(path, context)?);
				write_lock.insert(file_owned, model.clone());
				Ok(model)
			}
		}
	}*/
}
