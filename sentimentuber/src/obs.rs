use crate::cli::Config;

use std::path::Path;

use tokio::runtime::Runtime;
use tokio::runtime::Builder;

use obws::requests::inputs::SetSettings;
use obws::requests::scene_items::Source;
use obws::responses::sources::SourceId;
use obws::Client;

use serde_json::json;

pub struct OBSController {
	client: Client,
	image_source_id: SourceId,
	runtime: Runtime
}

impl OBSController {
	pub fn new(config: &Config) -> Result<Self, obws::error::Error> {
		let runtime = Builder::new_multi_thread()
	        .worker_threads(1)
	        .enable_all()
	        .build()
	        .unwrap();

		let ip = config.obs_ip.clone();
	    let password = config.obs_password.clone();
	    let port = config.obs_port;
	    let client = runtime.block_on(Client::connect(ip, port, Some(password)))?;
	    let image_source_name = config.obs_source_name.clone();
	    let image_source_id = runtime.block_on(get_image_scene_item(&client, &image_source_name))?;

		Ok(OBSController {
			client,
			image_source_id,
			runtime
		})
	}

	pub fn swap_image_to(&self, new_file_path: &str) -> Result<(), obws::error::Error>{
		let future = swap_obs_image_to(&self.image_source_id, new_file_path, &self.client);
		self.runtime.block_on(future)?;
		Ok(())
	}

}

async fn get_image_scene_item(client: &Client, image_source_name: &str) -> Result<SourceId, obws::error::Error> {
    let scenes_struct = client.scenes().list().await?;
    let test_scene = scenes_struct
        .scenes
        .iter()
        .find(|scene| scene.id.name.contains("SentimentTuber")) //TODO deal with this
        .expect("Could not find OBS scene by name");

    let items_in_scene = client
        .scene_items()
        .list(test_scene.id.clone().into())
        .await?;
    let image_source = items_in_scene
        .iter()
        .find(|item| {
            item.source_name.contains(image_source_name)
        })
        .expect("No image source found in OBS scene for the avatar");

    let source_id = client
        .scene_items()
        .source(Source {
            scene: test_scene.id.clone().into(),
            item_id: image_source.id,
        })
        .await?;

    Ok(source_id)
}

async fn swap_obs_image_to(
    source_id: &SourceId,
    new_file_path: &str,
    client: &Client,
) -> Result<(), obws::error::Error> {
    let path = Path::new(new_file_path);
    // Convert io err to fake obws error for now.
    let absolute = Path::canonicalize(path).map_err(|e| {
    	obws::error::Error::Api {
    		code: obws::responses::StatusCode::Unknown,
    		message: Option::from(e.to_string())
    	}
    })?;
    let setting = json!({"file": absolute});
    client
        .inputs()
        .set_settings(SetSettings {
            input: (&*source_id.name).into(),
            settings: &setting,
            overlay: Some(true),
        })
        .await?;

    Ok(())
}