use flexi_logger::{Logger, WriteMode, Criterion, Naming,FileSpec, Cleanup};
use std::path::PathBuf;


pub fn init_logger(level: &str, config_dir: PathBuf) -> anyhow::Result<()> {

  Logger::try_with_str(level)?
    .log_to_file(FileSpec::default().directory(config_dir).basename("log"))
    .write_mode(WriteMode::BufferAndFlush)
    .format(flexi_logger::detailed_format)
    .rotate(
      Criterion::Size(10 * 1024 * 1024), // 10MB でローテーション
      Naming::Timestamps,                // ファイル名にタイムスタンプを付与
      Cleanup::KeepLogFiles(5),          // 最新5ファイルを保持
    )
    .start()?;
  Ok(())
}