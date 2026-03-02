//! Async volume related tests

#![cfg(feature = "async")]

mod utils;

#[test]
fn async_open_all_volumes() {
    pollster::block_on(async {
        let time_source = utils::make_time_source();
        let disk = utils::make_async_block_device(utils::DISK_SOURCE).unwrap();
        let mut volume_mgr: embedded_sdmmc::AsyncVolumeManager<
            utils::AsyncRamDisk<Vec<u8>>,
            utils::TestTimeSource,
            4,
            4,
            2,
        > = embedded_sdmmc::AsyncVolumeManager::new_with_limits(disk, time_source, 0x1000_0000);

        // Open Volume 0
        let fat16_volume = volume_mgr
            .open_raw_volume(embedded_sdmmc::VolumeIdx(0))
            .await
            .expect("open volume 0");

        // Fail to Open Volume 0 again
        assert!(matches!(
            volume_mgr
                .open_raw_volume(embedded_sdmmc::VolumeIdx(0))
                .await,
            Err(embedded_sdmmc::Error::VolumeAlreadyOpen)
        ));

        volume_mgr
            .close_volume(fat16_volume)
            .await
            .expect("close fat16");

        // Open Volume 1
        let fat32_volume = volume_mgr
            .open_raw_volume(embedded_sdmmc::VolumeIdx(1))
            .await
            .expect("open volume 1");

        // Fail to Volume 1 again
        assert!(matches!(
            volume_mgr
                .open_raw_volume(embedded_sdmmc::VolumeIdx(1))
                .await,
            Err(embedded_sdmmc::Error::VolumeAlreadyOpen)
        ));

        // Open Volume 0 again
        let fat16_volume = volume_mgr
            .open_raw_volume(embedded_sdmmc::VolumeIdx(0))
            .await
            .expect("open volume 0");

        // Open any volume - too many volumes (0 and 1 are open)
        assert!(matches!(
            volume_mgr
                .open_raw_volume(embedded_sdmmc::VolumeIdx(0))
                .await,
            Err(embedded_sdmmc::Error::TooManyOpenVolumes)
        ));

        volume_mgr
            .close_volume(fat16_volume)
            .await
            .expect("close fat16");
        volume_mgr
            .close_volume(fat32_volume)
            .await
            .expect("close fat32");

        // This isn't a valid volume
        assert!(matches!(
            volume_mgr
                .open_raw_volume(embedded_sdmmc::VolumeIdx(2))
                .await,
            Err(embedded_sdmmc::Error::FormatError(_e))
        ));

        // This isn't a valid volume
        assert!(matches!(
            volume_mgr
                .open_raw_volume(embedded_sdmmc::VolumeIdx(3))
                .await,
            Err(embedded_sdmmc::Error::FormatError(_e))
        ));

        // This isn't a valid volume
        assert!(matches!(
            volume_mgr
                .open_raw_volume(embedded_sdmmc::VolumeIdx(9))
                .await,
            Err(embedded_sdmmc::Error::NoSuchVolume)
        ));

        // Re-open volume 1 to test dir-prevents-close
        let fat32_volume = volume_mgr
            .open_raw_volume(embedded_sdmmc::VolumeIdx(1))
            .await
            .expect("open volume 1");

        let _root_dir = volume_mgr.open_root_dir(fat32_volume).expect("Open dir");

        assert!(matches!(
            volume_mgr.close_volume(fat32_volume).await,
            Err(embedded_sdmmc::Error::VolumeStillInUse)
        ));
    });
}

#[test]
fn async_close_volume_too_early() {
    pollster::block_on(async {
        let time_source = utils::make_time_source();
        let disk = utils::make_async_block_device(utils::DISK_SOURCE).unwrap();
        let mut volume_mgr = embedded_sdmmc::AsyncVolumeManager::new(disk, time_source);

        let volume = volume_mgr
            .open_raw_volume(embedded_sdmmc::VolumeIdx(0))
            .await
            .expect("open volume 0");
        let root_dir = volume_mgr.open_root_dir(volume).expect("open root dir");

        // Dir open
        assert!(matches!(
            volume_mgr.close_volume(volume).await,
            Err(embedded_sdmmc::Error::VolumeStillInUse)
        ));

        let _test_file = volume_mgr
            .open_file_in_dir(root_dir, "64MB.DAT", embedded_sdmmc::Mode::ReadOnly)
            .await
            .expect("open test file");

        volume_mgr.close_dir(root_dir).unwrap();

        // File open, not dir open
        assert!(matches!(
            volume_mgr.close_volume(volume).await,
            Err(embedded_sdmmc::Error::VolumeStillInUse)
        ));
    });
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
