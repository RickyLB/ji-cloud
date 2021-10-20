use structopt::StructOpt;
use std::path::{Path, PathBuf};
use shared::config::RemoteTarget;

#[derive(Debug, StructOpt)]
#[structopt(name = "ji tap transcoder", about = "ji tap downloader/transcoder")]
pub struct Opts {
    #[structopt(long, default_value="https://d24o39yp3ttic8.cloudfront.net/106BBB0E-4966-4336-AB2D-7B210257646C/game.json")]
    pub game_json_url: String,

    #[structopt(long, default_value="D:\\Dropbox (Jewish Interactive)\\ji-cloud-media\\legacy\\examples", parse(from_os_str))]
    pub dest_base_path: PathBuf,

    #[structopt(long, default_value="json", parse(from_os_str))]
    pub dest_json_dir: PathBuf,

    #[structopt(long, default_value="media", parse(from_os_str))]
    pub dest_media_dir: PathBuf,

    /// debug mode 
    #[structopt(long, parse(try_from_str), default_value = "false")]
    pub debug: bool,

    // show output 
    #[structopt(short, long, parse(try_from_str), default_value = "true")]
    pub verbose: bool,
    
    /// download media 
    #[structopt(long, parse(try_from_str), default_value = "false")]
    pub download_media: bool,

    /// write json
    #[structopt(long, parse(try_from_str), default_value = "true")]
    pub write_json: bool,
}

impl Opts {
    pub fn sanitize(&mut self) {
        if self.debug {
            //log::warn!("sanitization: forcing dry_run since debug is true");
            //self.dry_run = true;
            //self.remote_target = "local".to_string();
        } 
    }

}