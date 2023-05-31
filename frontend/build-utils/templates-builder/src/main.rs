mod options;

use dotenv::dotenv;
use simplelog::*;
use options::Opts;
use structopt::StructOpt;
use tera::{Tera, Context};
use std::fs;
use std::path::PathBuf;

fn main() {
    dotenv().ok();
    let opts = Opts::from_args();
    init_logger(&opts);

    let dest = &opts.get_output_path();
    if opts.clean && dest.exists() {
        fs::remove_dir_all(dest).unwrap_ji();
    }


    let mut context = Context::new();
    context.insert("MEDIA_UI", &opts.get_remote_target().media_ui_url());

    let mut tera = get_tera(&opts.get_base_template_path());

    if opts.demo {
        tera.extend(&get_tera(&opts.get_demo_template_path())).unwrap_ji();
    }
    tera.build_inheritance_chains().unwrap_ji();
    tera.check_macro_files().unwrap_ji();


    let macro_names:Vec<&str> =
        tera.templates
            .values()
            .fold(Vec::new(), |mut acc, template| {
                if template.macros.len() != 0{
                    acc.push(&template.name);
                }
                acc
            });

    for name in tera.templates.keys()
        .filter(|name| {
            let name:&str = name;
            let valid = !macro_names.contains(&name);
            if !valid {
                log::info!("FILTERING OUT {}", name);
            }
            valid
        })
    {
        log::info!("rendering {}", name);

        let output_string = tera.render(name, &context).unwrap_ji();

        let output_path = dest.join(name);
        if !output_path.exists() {
            fs::create_dir_all(output_path.parent().unwrap_ji()).unwrap_ji();
        }
        fs::write(output_path, output_string).unwrap_ji();
    }
}


fn get_tera(source:&PathBuf) -> Tera {
    let source = source.join("**").join("*.html");
    let source = source.into_os_string().into_string().expect("couldn't get os string");

    let mut tera = match Tera::parse(&source) {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };

    tera.autoescape_on(vec![]);

    tera
}


fn init_logger(opts:&Opts) {
    if opts.verbose {
        CombinedLogger::init(vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        ])
        .unwrap_ji();
    } else {
        CombinedLogger::init(vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed),
        ])
        .unwrap_ji();
    }
}
