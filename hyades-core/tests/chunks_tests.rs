use hyades_core::chunk::{Data, Init, ParamType};

#[test]
fn test_init_conversion() {
    let buf = vec![
        1u8, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, // optional params
        0, 7, 0, 4, 0, 1, 0, 1,
    ];
    let chunk = Init::from(buf);
    assert!(chunk.optional_params.is_some());
    let params = chunk.optional_params.unwrap();
    assert!(params.len() == 1);
    let param = &params[0];
    assert!(param.param_type == ParamType::StateCookie);
    assert!(param.value == vec![0, 1, 0, 1]);
}

#[test]
fn test_init_conversion_2() {
    let buf = vec![
        1u8, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1,
        // optional params
        // param 1
        0, 7, 0, 4, 0, 1, 0, 1, // param 2
        0, 11, 0, 4, 0, 1, 0, 1,
    ];
    let chunk = Init::from(buf);
    assert!(chunk.optional_params.is_some());
    let params = chunk.optional_params.unwrap();
    assert!(params.len() == 2);
    let param = &params[1];
    assert!(param.param_type == ParamType::HostNameAddr);
    assert!(param.value == vec![0, 1, 0, 1]);
}

#[test]
fn test_init_conversion_with_no_params() {
    let buf = vec![
        1u8, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1,
        // no optional params
    ];
    let chunk = Init::from(buf);
    assert!(chunk.optional_params.is_none());
}

#[test]
fn test_data_chunk() {
    let chunk = Data::new(0, 1, 1, 0, true, false, vec![1, 2, 3]);
    assert!(chunk.data.len() == 4);
    assert!(chunk.data == vec![1, 2, 3, 0]);

    let chunk = Data::new(0, 1, 1, 0, true, false, vec![1, 2, 3, 4]);
    assert!(chunk.data.len() == 4);
    assert!(chunk.data == vec![1, 2, 3, 4]);

    let chunk = Data::new(0, 1, 1, 0, true, false, vec![1, 2, 3, 4]);
    assert!(chunk.data.len() == 4);
    assert!(chunk.data == vec![1, 2, 3, 4]);
}
