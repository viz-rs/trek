use serde::{de::value::Error, Deserialize};
use trek_core::parameters::Parameters;

#[test]
fn new_parameters() {
    #[derive(Debug, Deserialize)]
    struct MyStruct {
        user_id: usize,
        photo_id: Option<usize>,
    }

    // struct
    let p = Parameters::new(vec![("user_id", "233"), ("photo_id", "377")]);
    let s: MyStruct = p.params().unwrap();
    assert_eq!(s.user_id, 233);
    assert_eq!(s.photo_id.unwrap(), 377);

    // struct
    let p = Parameters::new(vec![("user_id", "233")]);
    let s: MyStruct = p.params().unwrap();
    assert_eq!(s.user_id, 233);
    assert_eq!(s.photo_id, None);

    // seq
    let p = Parameters::new(vec![("user_id", "233"), ("photo_id", "377")]);
    let s: Vec<String> = p.params().unwrap();
    assert_eq!(s, vec!["233", "377"]);

    // seq
    let p = Parameters::new(vec![("user_id", "233"), ("photo_id", "377")]);
    let s: [usize; 2] = p.params().unwrap();
    assert_eq!(s, [233, 377]);

    // tuple
    let p = Parameters::new(vec![("key", "age"), ("value", "32")]);
    let s: (String, String) = p.params().unwrap();
    assert_eq!(s.0, "age");
    assert_eq!(s.1, "32");

    // tuple
    let p = Parameters::new(vec![("key", "age"), ("value", "32")]);
    let s: (String, usize) = p.params().unwrap();
    assert_eq!(s.0, "age");
    assert_eq!(s.1, 32);

    // struct_tuple
    #[derive(Debug, Deserialize)]
    struct MyStructTuple(String, u32);

    let p = Parameters::new(vec![("key", "age"), ("value", "32")]);
    let s: MyStructTuple = p.params().unwrap();
    assert_eq!(s.0, "age");
    assert_eq!(s.1, 32);

    #[derive(Debug, Deserialize)]
    struct MyStruct2 {
        key: String,
        value: usize,
    }

    let p = Parameters::new(vec![("key", "age"), ("value", "32")]);
    let s: MyStruct2 = p.params().unwrap();
    assert_eq!(s.key, "age");
    assert_eq!(s.value, 32);

    let p = Parameters::new(vec![("id", "32")]);
    let s: i8 = p.params().unwrap();
    assert_eq!(s, 32);

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum MyEnum {
        Val1,
        Val2,
    }

    let p = Parameters::new(vec![("val", "val1")]);
    let s: MyEnum = p.params().unwrap();
    assert_eq!(s, MyEnum::Val1);

    let p = Parameters::new(vec![("val", "val2")]);
    let s: MyEnum = p.params().unwrap();
    assert_eq!(s, MyEnum::Val2);

    let p = Parameters::new(vec![("val1", "val2"), ("val2", "val1")]);
    let s: (MyEnum, MyEnum) = p.params().unwrap();
    assert_eq!(s.0, MyEnum::Val2);
    assert_eq!(s.1, MyEnum::Val1);

    #[derive(Debug, Deserialize)]
    struct MyStructEnum {
        val: MyEnum,
    }

    let p = Parameters::new(vec![("val", "val1")]);
    let s: MyStructEnum = p.params().unwrap();
    assert_eq!(s.val, MyEnum::Val1);

    let p = Parameters::new(vec![("val", "val3")]);
    let s: Result<MyEnum, Error> = p.params();
    assert!(s.is_err());
    assert!(format!("{:?}", s).contains("unknown variant `val3`, expected `val1` or `val2`"));
}
