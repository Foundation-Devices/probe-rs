use crate::common_options::{FlashOptions, OperationError};
use crate::{
    indicatif::{MultiProgress, ProgressBar, ProgressStyle},
    logging,
};

use std::{path::Path, sync::Arc, time::Instant};

use colored::Colorize;
use probe_rs::{
    flashing::{DownloadOptions, FlashLoader, FlashProgress, ProgressEvent},
    Session,
};

/// Performs the flash download with the given loader. Ensure that the loader has the data to load already stored.
/// This function also manages the update and display of progress bars.
pub fn run_flash_download(
    session: &mut Session,
    path: &Path,
    opt: &FlashOptions,
    loader: FlashLoader,
    do_chip_erase: bool,
) -> Result<(), OperationError> {
    // Start timer.
    let instant = Instant::now();

    let mut download_option = DownloadOptions::default();
    download_option.keep_unwritten_bytes = opt.restore_unwritten;
    download_option.dry_run = opt.probe_options.dry_run;
    download_option.do_chip_erase = do_chip_erase;
    download_option.disable_double_buffering = opt.disable_double_buffering;

    if !opt.disable_progressbars {
        // Create progress bars.
        let multi_progress = MultiProgress::new();
        let style = ProgressStyle::default_bar()
                    .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈✔")
                    .progress_chars("##-")
                    .template("{msg:.green.bold} {spinner} [{elapsed_precise}] [{wide_bar}] {bytes:>8}/{total_bytes:>8} @ {bytes_per_sec:>10} (eta {eta:3})");

        // Create a new progress bar for the fill progress if filling is enabled.
        let fill_progress = if opt.restore_unwritten {
            let fill_progress = Arc::new(multi_progress.add(ProgressBar::new(0)));
            fill_progress.set_style(style.clone());
            fill_progress.set_message("     Reading flash  ");
            Some(fill_progress)
        } else {
            None
        };

        // Create a new progress bar for the erase progress.
        let erase_progress = Arc::new(multi_progress.add(ProgressBar::new(0)));
        {
            logging::set_progress_bar(erase_progress.clone());
        }
        erase_progress.set_style(style.clone());
        erase_progress.set_message("     Erasing sectors");

        // Create a new progress bar for the program progress.
        let program_progress = multi_progress.add(ProgressBar::new(0));
        program_progress.set_style(style);
        program_progress.set_message(" Programming pages  ");

        // Register callback to update the progress.

        // Make the multi progresses print.
        // indicatif requires this in a separate thread as this join is a blocking op,
        // but is required for printing multiprogress.
        let progress_thread_handle = std::thread::spawn(move || {
            multi_progress.join().unwrap();
        });

        loader.commit(session, download_option, &mut |_| false).map_err(|error| {
            OperationError::FlashingFailed {
                source: error,
                target: session.target().clone(),
                target_spec: opt.probe_options.chip.clone(),
                path: path.to_path_buf(),
            }
        })?;

        // We don't care if we cannot join this thread.
        let _ = progress_thread_handle.join();
    } else {
        loader.commit(session, download_option, &mut |_| false).map_err(|error| {
            OperationError::FlashingFailed {
                source: error,
                target: session.target().clone(),
                target_spec: opt.probe_options.chip.clone(),
                path: path.to_path_buf(),
            }
        })?;
    }

    // Stop timer.
    let elapsed = instant.elapsed();
    logging::println(format!(
        "    {} in {}s",
        "Finished".green().bold(),
        elapsed.as_millis() as f32 / 1000.0,
    ));

    Ok(())
}
