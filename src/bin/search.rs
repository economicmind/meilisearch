use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

use heed::EnvOpenOptions;
use structopt::StructOpt;
use mega_mini_indexer::{Index, BEU32};

#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug, StructOpt)]
#[structopt(name = "mm-search", about = "The server side of the MMI project.")]
struct Opt {
    /// The database path where the database is located.
    /// It is created if it doesn't already exist.
    #[structopt(long = "db", parse(from_os_str))]
    database: PathBuf,

    /// The query string to search for (doesn't support prefix search yet).
    query: String,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    std::fs::create_dir_all(&opt.database)?;
    let env = EnvOpenOptions::new()
        .map_size(100 * 1024 * 1024 * 1024) // 100 GB
        .max_readers(10)
        .max_dbs(5)
        .open(opt.database)?;

    let index = Index::new(&env)?;

    let before = Instant::now();
    let rtxn = env.read_txn()?;

    let documents_ids = index.search(&rtxn, &opt.query)?;
    let headers = match index.headers(&rtxn)? {
        Some(headers) => headers,
        None => return Ok(()),
    };

    let mut stdout = io::stdout();
    stdout.write_all(&headers)?;

    for id in &documents_ids {
        if let Some(content) = index.documents.get(&rtxn, &BEU32::new(*id))? {
            stdout.write_all(&content)?;
        }
    }

    eprintln!("Took {:.02?} to find {} documents", before.elapsed(), documents_ids.len());

    Ok(())
}
