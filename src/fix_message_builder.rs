use std::collections::BTreeMap;

use crate::{
    split_fix_to_tags,
    utils::{bytes_to_fix_string, calculate_check_sum, compile_fix_chunk},
    FixSerializeError,
};

pub const FIX_VERSION: &[u8] = b"8";
pub const FIX_BODY_LEN: &[u8] = b"9";
pub const FIX_CHECK_SUM: &[u8] = b"10";
pub const FIX_MESSAGE_TYPE: &[u8] = b"35";

#[derive(Clone)]
pub struct FixMessageBuilder {
    fix_version: Vec<u8>,
    message_type: Vec<u8>,
    data: BTreeMap<u32, (Vec<u8>, Vec<u8>)>,
}

impl FixMessageBuilder {
    pub fn from_bytes(
        payload: &[u8],
        check_sum_validation: bool,
    ) -> Result<Self, FixSerializeError> {
        let tags = split_fix_to_tags(payload);

        let version = tags.get(FIX_VERSION);
        let message_type = tags.get(FIX_MESSAGE_TYPE);
        let source_check_sum = tags.get(FIX_CHECK_SUM);

        if version.is_none() {
            println!(
                "Tag not found: {:?}. Str: {}",
                payload.clone(),
                String::from_utf8(payload.clone().to_vec()).unwrap()
            );

            return Err(FixSerializeError::VersionTagNotFoundInSource);
        }

        if message_type.is_none() {
            return Err(FixSerializeError::MessageTypeTagNotFoundInSource);
        }

        if check_sum_validation == true && source_check_sum.is_none() {
            return Err(FixSerializeError::CheckSumTagNotFoundInSource);
        }

        let mut result = Self {
            fix_version: version.unwrap().clone(),
            message_type: message_type.unwrap().clone(),
            data: BTreeMap::new(),
        };

        let to_skip = vec![FIX_BODY_LEN, FIX_VERSION, FIX_CHECK_SUM];

        for (tag, value) in &tags {
            if to_skip.contains(&tag.as_slice()) {
                continue;
            }

            result.with_value_as_bytes(tag.clone(), value.clone());
        }

        if check_sum_validation {
            if source_check_sum.unwrap() != &result.calculate_check_sum().as_bytes().to_vec() {
                return Err(FixSerializeError::InvalidCheckSum);
            }
        }

        return Ok(result);
    }

    pub fn new(version: &str, message_type: &str) -> Self {
        return Self {
            fix_version: version.as_bytes().to_vec(),
            message_type: message_type.as_bytes().to_vec(),
            data: BTreeMap::new(),
        };
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        return self.compile_message();
    }

    pub fn get_value(&self, key: Vec<u8>) -> Option<&Vec<u8>> {
        for (inner_key, value) in self.data.values() {
            if inner_key == &key {
                return Some(value);
            }
        }

        return None;
    }

    pub fn get_message_type(&self) -> &Vec<u8> {
        return &self.message_type;
    }

    pub fn get_value_as_string(&self, key: Vec<u8>) -> Option<String> {
        for (inner_key, value) in self.data.values() {
            if inner_key == &key {
                return Some(String::from_utf8(value.clone()).unwrap());
            }
        }

        return None;
    }

    pub fn get_value_string(&self, key: &str) -> Option<String> {
        for (inner_key, value) in self.data.values() {
            if inner_key == &key.as_bytes() {
                return Some(String::from_utf8(value.clone()).unwrap());
            }
        }

        return None;
    }

    pub fn with_value(&mut self, key: i32, value: &str) {
        self.data.insert(
            self.get_last_index(),
            (
                key.to_string().as_bytes().to_vec(),
                value.as_bytes().to_vec(),
            ),
        );
    }

    fn with_value_as_bytes(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(self.get_last_index(), (key, value));
    }

    fn compile_message(&self) -> Vec<u8> {
        let mut result = compile_fix_chunk(FIX_VERSION, &self.fix_version);

        let (body_len, body) = self.compile_body();

        result.extend_from_slice(&compile_fix_chunk(
            FIX_BODY_LEN,
            body_len.to_string().as_bytes(),
        ));
        result.extend_from_slice(&body);

        result.extend_from_slice(&compile_fix_chunk(
            FIX_CHECK_SUM,
            calculate_check_sum(&result).as_bytes(),
        ));

        return result;
    }

    fn calculate_check_sum(&self) -> String {
        let mut result = compile_fix_chunk(FIX_VERSION, &self.fix_version);

        let (body_len, body) = self.compile_body();

        result.extend_from_slice(&compile_fix_chunk(
            FIX_BODY_LEN,
            body_len.to_string().as_bytes(),
        ));
        result.extend_from_slice(&body);

        return calculate_check_sum(&result);
    }

    fn compile_body(&self) -> (usize, Vec<u8>) {
        let mut body: Vec<u8> = compile_fix_chunk(FIX_MESSAGE_TYPE, &self.message_type);

        for (_, (key, value)) in &self.data {
            let data_to_insert = compile_fix_chunk(key, value);
            body.extend_from_slice(&data_to_insert)
        }

        return (body.len(), body);
    }

    fn get_last_index(&self) -> u32 {
        return match self.data.keys().last() {
            Some(last_index) => last_index.clone() + 1,
            None => 0,
        };
    }
}

impl ToString for FixMessageBuilder {
    fn to_string(&self) -> String {
        let bytes = self.compile_message();
        return bytes_to_fix_string(&bytes);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_fix_string() {
        let fix_string = "8=FIX.4.4|9=75|35=A|34=1092|49=TESTBUY1|52=20180920-18:24:59.643|56=TESTSELL1|98=0|108=60|10=178|";

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(34, &"1092".to_string());
        fix_builder.with_value(49, &"TESTBUY1".to_string());
        fix_builder.with_value(52, &"20180920-18:24:59.643".to_string());
        fix_builder.with_value(56, &"TESTSELL1".to_string());
        fix_builder.with_value(98, &"0".to_string());
        fix_builder.with_value(108, &"60".to_string());

        let fix_to_assert: String = fix_builder.to_string();

        assert_eq!(fix_string, &fix_to_assert);
    }

    #[test]
    fn test_invalid_fix_no_version() {
        let fix_string =
            b"9=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=178";

        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::VersionTagNotFoundInSource as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_invalid_fix_no_message_type() {
        let fix_string =
            b"8=FIX.4.49=75108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=178";
        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::MessageTypeTagNotFoundInSource as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_no_check_sum_with_validation() {
        let fix_string =
            b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0";
        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::CheckSumTagNotFoundInSource as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_no_check_sum_with_disabled_validation() {
        let fix_string =
            b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0";
        let builder = FixMessageBuilder::from_bytes(fix_string, false);

        assert_eq!(false, builder.is_err());
    }
    #[test]
    fn test_invalid_fix_check_sum_with_disabled_validation() {
        let fix_string = b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=188";
        let builder = FixMessageBuilder::from_bytes(fix_string, false);

        assert_eq!(false, builder.is_err());
    }

    #[test]
    fn test_invalid_fix_check_sum() {
        let fix_string = b"8=FIX.4.49=7535=A108=6034=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=010=188";
        let builder = FixMessageBuilder::from_bytes(fix_string, true);

        assert_eq!(true, builder.is_err());
        assert_eq!(
            FixSerializeError::InvalidCheckSum as i32,
            builder.err().unwrap() as i32
        );
    }

    #[test]
    fn test_to_bytes() {
        let fix_string = b"8=FIX.4.49=7535=A34=109249=TESTBUY152=20180920-18:24:59.64356=TESTSELL198=0108=6010=178";

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(34, &"1092".to_string());
        fix_builder.with_value(49, &"TESTBUY1".to_string());
        fix_builder.with_value(52, &"20180920-18:24:59.643".to_string());
        fix_builder.with_value(56, &"TESTSELL1".to_string());
        fix_builder.with_value(98, &"0".to_string());
        fix_builder.with_value(108, &"60".to_string());

        let fix_to_assert = fix_builder.as_bytes();

        assert_eq!(fix_string, fix_to_assert.as_slice());
    }
}
