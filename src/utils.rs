use crate::Result;
use serde_json::Value;
use std::convert::TryInto;
use std::env::var;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

pub fn create_json(value: &mut serde_json::Value) -> Result<String> {
  let uuid = Uuid::new_v4().to_string();

  let payload = value.as_object_mut().expect("payload must be an object");
  payload.insert("nonce".to_string(), Value::String(uuid));

  // TODO: handle error
  Ok(serde_json::to_string(&payload)?)
}

// Re-implement some packing methods in Rust
pub fn pack(opcode: u32, data_len: u32) -> Result<Vec<u8>> {
  let mut bytes = Vec::new();

  for byte_array in &[opcode.to_le_bytes(), data_len.to_le_bytes()] {
    bytes.extend_from_slice(byte_array);
  }

  Ok(bytes)
}

pub fn unpack(data: Vec<u8>) -> Result<(u32, u32)> {
  let data = data.as_slice();
  let (opcode, header) = data.split_at(std::mem::size_of::<u32>());

  let opcode = u32::from_le_bytes(opcode.try_into()?);
  let header = u32::from_le_bytes(header.try_into()?);

  Ok((opcode, header))
}

/// Finds the discord IPC pipe path
pub fn get_pipe_path() -> Option<PathBuf> {
  #[cfg(target_os = "windows")]
  let possible_paths = vec![r"\\?\pipe\discord-ipc-".to_path_buf()];

  #[cfg(target_family = "unix")]
  let mut possible_paths = vec!["/tmp/discord-ipc-".to_string()];

  if let Ok(runtime_dir) = var("XDG_RUNTIME_DIR") {
    // Flatpak installed Discord
    possible_paths.push(runtime_dir.clone() + "/app/com.discordapp.Discord/discord-ipc-");
    // Non-Flatpak installed Discord
    possible_paths.push(runtime_dir + "/discord-ipc-");
  }

  for i in 0..10 {
    for p in &possible_paths {
      let path: String = format!("{}{}", p, i);

      if Path::new(&path).exists() {
        return Some(Path::new(&path).to_path_buf());
      }
    }
  }

  None
}
