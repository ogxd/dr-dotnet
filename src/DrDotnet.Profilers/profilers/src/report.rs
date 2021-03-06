use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Local};

use std::path::{PathBuf, Path};
use std::io::BufWriter;
use std::fs::File;
use std::io::Write;

use crate::profilers::ProfilerData;

// A Session refers to a profiling session. In conists in a process, a profiler, and a timestamp at which the profiling was done.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    session_id: Uuid,
    process_name: String,
    timestamp: DateTime<Local>,
    profiler: ProfilerData,
}

impl Session {

    // Returns a Session from its UID and ProfilerData.
    // If the Session report is not present on the disk, it will be written at the same time.
    pub fn get_session(session_id: Uuid, profiler: ProfilerData) -> Session {

        let process_name = std::env::current_exe().unwrap()
            .file_name().unwrap()
            .to_str().unwrap()
            .to_owned();

        let report = Session {
            session_id: session_id,
            process_name : process_name,
            profiler: profiler,
            timestamp: chrono::offset::Local::now()
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&report).unwrap();

        // Write session report
        let json_path = format!("{}/session.json", Session::get_directory(session_id));
        if !Path::exists(Path::new(&json_path)) {
            let mut session_stream = File::create(json_path).expect("Unable to create file");
            session_stream.write_all(json.as_bytes()).expect("Unable to write data");    
        }

        return report;
    }

    // Create a new report for a given Session, ready to be filled up.
    pub fn create_report(&self, filename: String) -> Report {
        let path = PathBuf::from(format!(r"{}/{}", Session::get_directory(self.session_id), filename));
        let file = File::create(&path).unwrap();
        return Report { writer: BufWriter::new(file) };
    }

    // Returns the directy path for this Session.
    pub fn get_directory(session_id: Uuid) -> String {
        let directory_path = format!(r"/dr-dotnet/{}", session_id.to_string());
        std::fs::create_dir_all(&directory_path);
        return directory_path;
    }
}

// A Session can contain several reports, which are usually files like markdown summaries or charts.
pub struct Report {
    pub writer: BufWriter<File>,
}

impl Report {
    pub fn write_line(&mut self, text: String) {
        self.writer.write(format!("{}\r\n", text).as_bytes()).unwrap();
    }

    pub fn new_line(&mut self) {
        self.writer.write(b"\r\n").unwrap();
    }
}