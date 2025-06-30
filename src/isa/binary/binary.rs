pub trait BinaryIO {
    fn to_binary(&self) -> Vec<u8>;
    fn from_binary(code: &Vec<u8>) -> Result<Self, String>
    where
        Self: Sized;
}

impl BinaryIO for u64 {
    fn to_binary(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    /// Load a u64 from a binary vector with little endian encoding
    fn from_binary(binary: &Vec<u8>) -> Result<Self, String> {
        if binary.len() != 8 {
            return Err(format!(
                "Invalid binary length: expected 8, got {}",
                binary.len()
            ));
        }

        Ok(binary
            .iter()
            .enumerate()
            .fold(0u64, |acc, (i, &byte)| acc | ((byte as u64) << (i * 8))))
    }
}

pub trait BinaryStringIO {
    fn to_binary_str(&self) -> String;
    fn from_binary_str(s: &str) -> Result<Self, String>
    where
        Self: Sized;
    fn from_binary_prog_file(path: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let file = std::fs::read_to_string(path).unwrap();
        let file = file.replace(" ", "").replace("\n", "");
        Self::from_binary_str(&file)
    }
}

impl BinaryStringIO for Vec<u8> {
    fn to_binary_str(&self) -> String {
        self.iter().map(|&byte| format!("{:08b}", byte)).collect()
    }

    fn from_binary_str(s: &str) -> Result<Self, String> {
        // Validate that the string only contains '0' and '1' characters
        if !s.chars().all(|c| c == '0' || c == '1') {
            return Err("Binary string must contain only '0' and '1' characters".to_string());
        }

        // Validate that the string length is a multiple of 8
        if s.len() % 8 != 0 {
            return Err(format!(
                "Binary string length ({}) must be a multiple of 8",
                s.len()
            ));
        }

        // Convert the string to bytes
        s.as_bytes()
            .chunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .enumerate()
                    .try_fold(0u8, |byte, (i, &c)| match c {
                        b'1' => Ok(byte | (1 << (7 - i))),
                        b'0' => Ok(byte),
                        _ => Err("Invalid character in binary string".to_string()),
                    })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_io() {
        let code = 0b1010101010101010101010101010101010101010101010101010101010101010;
        let binary = code.to_binary();
        let expected_binary = vec![
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010,
        ];
        assert_eq!(binary, expected_binary);
        let code_str = binary.to_binary_str();
        assert_eq!(
            code_str,
            "1010101010101010101010101010101010101010101010101010101010101010"
        );
    }
}
