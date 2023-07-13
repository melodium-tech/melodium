use melodium_repository::*;
use platforms::Platform;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let repo = &args[1];
    let pkg = &args[2];
    let toml = &args[3];

    let config = RepositoryConfig {
        repository_location: repo.into(),
        network: None,
    };

    let mut repo = Repository::new(config);
    repo.load_packages().unwrap();

    let (tech_pkg, global_pkg) = utils::cargo_toml(pkg, toml).unwrap();

    repo.add_package(tech_pkg.clone()).unwrap();
    repo.set_package_details(&global_pkg).unwrap();

    for element in args.iter().skip(4) {
        let element = element.split(':').collect::<Vec<_>>();

        let target = Platform::find(element[0]).unwrap();
        let qualif = match element[1] {
            "real" => technical::Availability::Real,
            "mock" => technical::Availability::Mock,
            _ => panic!("Unknown availability type"),
        };
        let path = PathBuf::from(element[2]);
        let hash = element[3];

        repo.set_platform_availability(
            &tech_pkg,
            target,
            &qualif,
            technical::Element {
                name: path.file_name().unwrap().to_string_lossy().to_string(),
                checksum: hash.to_string(),
            },
        )
        .unwrap();
        let location = repo
            .get_package_element_path(&tech_pkg, Some((target, &qualif)))
            .unwrap();

        std::fs::create_dir_all(location.parent().unwrap()).unwrap();
        std::fs::copy(path, location).unwrap();
    }
}
