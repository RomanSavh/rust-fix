use std::collections::HashMap;

pub const FIX_EQUALS: u8 = 0x3d;
pub const FIX_DELIMETR: u8 = 0x1;

pub fn calculate_check_sum(body: &[u8]) -> String {
    let mut sum = 0u8;
    for byte in body {
        sum = sum.wrapping_add(*byte);
    }

    return format!("{:0>3}", sum.to_string());
}

pub fn compile_fix_chunk(key: &[u8], value: &[u8]) -> Vec<u8>{
    let mut result: Vec<u8> = vec![];

    result.extend_from_slice(key);
    result.extend_from_slice(&vec![FIX_EQUALS]);
    result.extend_from_slice(&value);
    result.extend_from_slice(&vec![FIX_DELIMETR]);

    return result;
}

pub fn bytes_to_fix_string(data: &[u8]) -> String{
    let mut str = vec![];

    for byte in data{
        if byte == &FIX_DELIMETR{
            str.extend_from_slice(b"|");
        } else {
            str.push(*byte);
        }
    }

    return String::from_utf8(str).unwrap();
}

pub fn split_fix_to_tags(fix: &[u8]) -> HashMap<Vec<u8>, Vec<u8>>{
    let mut result = HashMap::new();
    let mut key_buffer = Vec::new();
    let mut value_buffer = Vec::new();
    let mut is_equals_raised = false;
    
    for byte in fix{
        if byte == &FIX_DELIMETR {
            result.insert(key_buffer.clone(), value_buffer.clone());
            key_buffer.clear();
            value_buffer.clear();
            is_equals_raised = false;
            continue;
        }

        if byte == &FIX_EQUALS {
            is_equals_raised = true;
            continue;
        }

        match is_equals_raised{
            true => value_buffer.push(byte.clone()),
            false => key_buffer.push(byte.clone()),
        };
    }

    return result;
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compile_fix_chunk() {
        let bytes = b"8=FIX.4.4";
        
        let key = b"8";
        let value = b"FIX.4.4";

        let result = compile_fix_chunk(key, value);

        assert_eq!(bytes, &result[..9]);
        assert_eq!(&vec![FIX_DELIMETR], &result[9..10]);
        
    }

    #[test]
    fn test_check_sum_calculation() {

        let check_sum = b"178";

        let test_body = build_test_body(vec![
            "8=FIX.4.4",
            "9=75",
            "35=A",
            "34=1092",
            "49=TESTBUY1",
            "52=20180920-18:24:59.643",
            "56=TESTSELL1",
            "98=0",
            "108=60"
        ]);

        let result = calculate_check_sum(&test_body);
        assert_eq!(check_sum, result.as_bytes());
    }

    #[test]
    fn test_bytes_to_fix_string(){
        let fix_text_string = "8=FIX.4.4|9=75|35=A|";
        let test_fix_message = build_test_body(vec![
            "8=FIX.4.4",
            "9=75",
            "35=A"
        ]);

        let string_message = bytes_to_fix_string(&test_fix_message);

        assert_eq!(string_message.as_str(), fix_text_string);
    }

    fn build_test_body(data: Vec<&str>) -> Vec<u8>{
        let mut result = vec![];
        for itm in data{
            result.extend_from_slice(itm.as_bytes());
            result.extend_from_slice(&vec![FIX_DELIMETR]);
        }

        return result;
    }
}