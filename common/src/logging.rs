use log::LevelFilter;

pub fn init_fern_logger()->Result<(), fern::InitError>{
    init_fern_logger_with_log_level(None)
}

pub fn init_fern_logger_with_log_level(level: Option<LevelFilter>)->Result<(), fern::InitError>{
    let fern_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S.%f]"),
                record.level(),
                message
            ))
        })
        .level(level.unwrap_or(LevelFilter::Trace))
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?);

    fern_logger.apply()?;

    Ok(())
}