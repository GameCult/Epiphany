use anyhow::{Context, Result, anyhow, bail};
use std::io::{Read, Write};
use uuid::Uuid;
use zeroize::Zeroize;

const MAGIC: &[u8; 8] = b"EPHWCOV\0";
const FRAME_LEN: usize = MAGIC.len() + 16 + 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageProcessBootstrap {
    pub launch_id: Uuid,
    pub provider_signing_seed: [u8; 32],
}

impl Drop for WorkspaceCoverageProcessBootstrap {
    fn drop(&mut self) {
        self.provider_signing_seed.zeroize();
    }
}

pub fn write_workspace_coverage_process_bootstrap(
    mut writer: impl Write,
    frame: &WorkspaceCoverageProcessBootstrap,
) -> Result<()> {
    let mut bytes = [0_u8; FRAME_LEN];
    bytes[..MAGIC.len()].copy_from_slice(MAGIC);
    bytes[MAGIC.len()..MAGIC.len() + 16].copy_from_slice(frame.launch_id.as_bytes());
    bytes[MAGIC.len() + 16..].copy_from_slice(&frame.provider_signing_seed);
    writer
        .write_all(&bytes)
        .context("failed to write workspace coverage bootstrap frame")?;
    writer
        .flush()
        .context("failed to flush workspace coverage bootstrap frame")
}

pub fn read_workspace_coverage_process_bootstrap(
    mut reader: impl Read,
) -> Result<WorkspaceCoverageProcessBootstrap> {
    let mut bytes = [0_u8; FRAME_LEN];
    reader
        .read_exact(&mut bytes)
        .context("workspace coverage bootstrap frame is truncated")?;
    let mut trailing = [0_u8; 1];
    if reader
        .read(&mut trailing)
        .context("failed to verify workspace coverage bootstrap EOF")?
        != 0
    {
        bail!("workspace coverage bootstrap contains trailing bytes");
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        bail!("workspace coverage bootstrap magic is invalid");
    }
    let launch_id = Uuid::from_slice(&bytes[MAGIC.len()..MAGIC.len() + 16])
        .map_err(|_| anyhow!("workspace coverage bootstrap launch id is invalid"))?;
    let provider_signing_seed = bytes[MAGIC.len() + 16..]
        .try_into()
        .map_err(|_| anyhow!("workspace coverage bootstrap seed length is invalid"))?;
    Ok(WorkspaceCoverageProcessBootstrap {
        launch_id,
        provider_signing_seed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_frame_round_trips_and_requires_eof() {
        let frame = WorkspaceCoverageProcessBootstrap {
            launch_id: Uuid::new_v4(),
            provider_signing_seed: [7; 32],
        };
        let mut encoded = Vec::new();
        write_workspace_coverage_process_bootstrap(&mut encoded, &frame).unwrap();
        assert_eq!(encoded.len(), FRAME_LEN);
        assert_eq!(
            read_workspace_coverage_process_bootstrap(encoded.as_slice()).unwrap(),
            frame
        );
        encoded.push(0);
        assert!(read_workspace_coverage_process_bootstrap(encoded.as_slice()).is_err());
    }

    #[test]
    fn truncated_frame_is_rejected() {
        assert!(read_workspace_coverage_process_bootstrap(&[0_u8; FRAME_LEN - 1][..]).is_err());
    }
}
