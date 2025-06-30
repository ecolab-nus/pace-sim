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

    fn from_binary(binary: &Vec<u8>) -> Result<Self, String> {
        if binary.len() != 8 {
            return Err(format!("Invalid binary length: {}", binary.len()));
        }
        let mut code: u64 = 0;
        for (i, c) in binary.iter().enumerate() {
            if *c == 1 {
                code |= 1 << (63 - i);
            }
        }
        Ok(code)
    }
}

pub trait BinaryStringIO {
    fn to_binary_str(&self) -> String;
    fn from_binary_str(s: &str) -> Result<Self, String>
    where
        Self: Sized;
}

impl BinaryStringIO for Vec<u8> {
    fn to_binary_str(&self) -> String {
        let mut s = String::new();
        for c in self {
            s.push_str(&c.to_string());
        }
        s
    }

    fn from_binary_str(s: &str) -> Result<Self, String> {
        let mut binary = Vec::new();
        // for each 8 characters, convert to u8
        for i in (0..s.len()).step_by(8) {
            let mut byte = 0;
            for j in 0..8 {
                if s.chars().nth(i + j).unwrap() == '1' {
                    byte |= 1 << (7 - j);
                }
            }
            binary.push(byte);
        }
        Ok(binary)
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::configuration::Configuration;

    use super::*;

    #[test]
    fn test_binary_io() {
        let code = 0b1010101010101010101010101010101010101010101010101010101010101010;
        let code_str = code.to_binary().to_binary_str();
        let code_binary = code.to_binary();
        assert_eq!(
            code_str,
            "1010101010101010101010101010101010101010101010101010101010101010"
        );
        assert_eq!(
            code_binary,
            vec![
                0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
                0b10101010,
            ]
        );
        assert_eq!(code_binary.len(), 8);
        assert_eq!(code_binary[0], 0b10101010);
        assert_eq!(code_binary[1], 0b10101010);
        assert_eq!(code_binary[2], 0b10101010);
        assert_eq!(code_binary[3], 0b10101010);
        assert_eq!(code_binary[4], 0b10101010);
        assert_eq!(code_binary[5], 0b10101010);
        assert_eq!(code_binary[6], 0b10101010);
        assert_eq!(code_binary[7], 0b10101010);
    }
}
