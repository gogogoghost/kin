use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    borrow::Borrow,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write, Seek},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

static webapps_root: &str = "/data/local/webapps";
static webapps_list_path: &str = "/data/local/webapps/webapps.json";

#[derive(Serialize, Deserialize)]
struct Manifest {
    name: String,
    version: String,
    origin: String,
}

#[derive(Serialize, Deserialize)]
struct AppItem {
    origin: String,
    installOrigin: String,
    manifestURL: String,
    appStatus: i32,
    receipts: Vec<String>,
    kind: String,
    installTime: u128,
    installState: String,
    removable: bool,
    id: String,
    basePath: String,
    localId: i64,
    sideloaded: bool,
    enabled: bool,
    blockedStatus: i32,
    name: String,
    csp: String,
    role: String,
    redirects: Option<String>,
    widgetPages: Vec<String>,
    userAgentInfo: String,
    installerAppId: i32,
    installerIsBrowser: bool,
    storeId: String,
    storeVersion: i32,
    downloading: bool,
    readyToApplyDownload: bool,
    oldVersion: String,
    additionalLanguages: AdditionalLanguage,
}

#[derive(Serialize, Deserialize)]
struct AdditionalLanguage {}

fn check_app(mut zip: ZipArchive<File>, zip_path: String) -> (ZipArchive<File>, String) {
    if let Ok(mut app_zip_file) = zip.by_name("application.zip") {
        let tmp_file = "/cache/kin_app.zip";
        let mut buf = Vec::new();
        app_zip_file.read_to_end(&mut buf).unwrap();
        let mut app_file = File::create(tmp_file).expect("Failed to create tmp file");
        app_file.write(&buf).expect("Failed to write to tmp file");
        app_file.flush().unwrap();
        return (
            zip::ZipArchive::new(File::open(tmp_file).unwrap()).expect("Failed to read inner application.zip"),
            tmp_file.to_owned(),
        );
    }
    return (zip, zip_path);
}

fn main() {
    let args: Vec<_> = env::args().collect();
    //check arg
    let file_path = args.get(1).expect("Please specify a file");
    //open file
    let file = fs::File::open(file_path).expect("Failed to open file");
    //open as zip
    let zip_file = zip::ZipArchive::new(file).expect("Failed to read as zip file");
    //if it contains application
    let (mut app_zip, app_zip_path) = check_app(zip_file, file_path.to_owned());

    let manifest_file = app_zip.by_name("manifest.webapp");
    if manifest_file.is_err() {
        panic!("Failed to find manifest.webapp")
    }
    let mut manifest_file = manifest_file.unwrap();
    let mut manifest_raw = Vec::new();
    manifest_file.read_to_end(&mut manifest_raw).unwrap();
    let manifest: Manifest =
        serde_json::from_slice::<Manifest>(&manifest_raw).expect("Failed to deserialize manifest");

    let app_id = &manifest.origin[6..];

    //read system manifest
    let mut app_list: Map<String, Value> = serde_json::from_str(
        fs::read_to_string(webapps_list_path)
            .expect("Failed to read webapp list file")
            .borrow(),
    )
    .expect("Failed to parse webapp list file");

    //check if it exists
    if app_list.contains_key(app_id) {
        panic!("This app is already installed")
    }

    //get max
    let mut max_local_id: i64 = 1;
    for (_, v) in &app_list {
        let obj = v.as_object().expect("Failed to parse app item");
        let local_id = obj["localId"].as_i64().expect("Failed to read localId");
        if local_id > max_local_id {
            max_local_id = local_id
        }
    }

    //generate
    let item = AppItem {
        origin: manifest.origin.to_owned(),
        installOrigin: manifest.origin.to_owned(),
        manifestURL: manifest.origin.to_owned() + "/manifest.webapp",
        appStatus: 3,
        receipts: Vec::new(),
        kind: "packaged".to_owned(),
        installTime: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        installState: "installed".to_owned(),
        removable: true,
        id: app_id.to_owned(),
        basePath: webapps_root.to_owned(),
        localId: max_local_id + 1,
        sideloaded: false,
        enabled: true,
        blockedStatus: 0,
        name: manifest.name,
        csp: "".to_owned(),
        role: "".to_owned(),
        redirects: None,
        widgetPages: Vec::new(),
        userAgentInfo: "".to_owned(),
        installerAppId: 0,
        installerIsBrowser: false,
        storeId: "".to_owned(),
        storeVersion: 0,
        downloading: false,
        readyToApplyDownload: false,
        oldVersion: manifest.version,
        additionalLanguages: AdditionalLanguage {},
    };

    //concat app list
    app_list.insert(app_id.to_owned(), serde_json::to_value(item).unwrap());

    //copy files
    let dir = webapps_root.to_owned() + "/" + app_id;
    fs::create_dir(&dir).expect("Failed to create app dir");
    fs::copy(app_zip_path, dir.to_owned() + "/application.zip")
        .expect("Failed to copy application zip");
    fs::write(dir + "/manifest.webapp", manifest_raw).expect("Failed to write manifest file");

    //update app list
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    app_list.serialize(&mut ser).unwrap();
    fs::write(webapps_list_path, buf).expect("Failed to over write app list file");
    println!("{} has installed",app_id)
}
