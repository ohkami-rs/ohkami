use ohkami_macros::JSON;
use ohkami_json::JSON;

fn main() {
    let user = User {
        id: 1,
        name: String::from("Taro")
    };

    let ser = user.serialize();
    println!("serialized: {ser}");

    let mut case = r#"{"id": 1, "name": "Taro"}"#.chars().peekable();
    let de = <User as JSON>::_deserialize(&mut case);

    let case_str = case.collect::<String>();
    match case_str.as_str() {
        "" => println!("ok"),
        _ => {
            println!("remained: {case_str}");
            return
        },
    }

    match de {
        None => println!("can't deserialize!!!"),
        Some(deserialized) => {
            assert_eq!(deserialized, user);
            println!("deserialized: {deserialized:?}");
        },
    }
}

#[derive(JSON, PartialEq, Debug)]
struct User {
    id:   u64,
    name: String,
}


// struct User {
//     id: u64,
//     name: String,
// }
// impl JSON for User {
//     fn serialize(&self) -> String {
//         let mut s = String::from("{");
//         s += "\"name\":";
//         s += &format!(r#"{}"#, self.name);
//         s += ",\"id\":";
//         s += &self.id.to_string();
//         s + "}"
//     }
//     fn _deserialize(string: &mut std::iter::Peekable<std::str::Chars>) -> Option<Self> {
//         let mut id = None;
//         let mut name = None;
//         string.next_if_eq(&'{')?;
//         loop {
//             match string.peek()? {
//                 '}' => {
//                     string.next();
//                     return (string.next().is_none() && id.is_some() && name.is_some())
//                         .then(|| User {
//                             id: id.unwrap(),
//                             name: name.unwrap(),
//                         });
//                 }
//                 _ => {
//                     match 'string: {
//                         string.next_if_eq(&'"')?;
//                         let mut ret = String::new();
//                         while let Some(ch) = string.next() {
//                             match ch {
//                                 '"' => break 'string Some(ret),
//                                 _ => ret.push(ch),
//                             }
//                         }
//                         None
//                     }?
//                         .as_str()
//                     {
//                         "id" => {
//                             string.next_if_eq(&':')?;
//                             string.next_if_eq(&' ');
//                             if id
//                                 .replace({
//                                     let mut int_str = String::new();
//                                     while let Some(ch) = string.peek() {
//                                         match ch {
//                                             '0'..='9' => int_str.push(string.next().unwrap()),
//                                             _ => return None,
//                                         }
//                                     }
//                                     int_str.parse::<u64>().ok()?
//                                 })
//                                 .is_some()
//                             {
//                                 return None;
//                             }
//                             string.next_if_eq(&',');
//                             string.next_if_eq(&' ');
//                         }
//                         "name" => {
//                             string.next_if_eq(&':')?;
//                             string.next_if_eq(&' ');
//                             if name
//                                 .replace(
//                                     'string: {
//                                         string.next_if_eq(&'"')?;
//                                         let mut ret = String::new();
//                                         while let Some(ch) = string.next() {
//                                             match ch {
//                                                 '"' => break 'string Some(ret),
//                                                 _ => ret.push(ch),
//                                             }
//                                         }
//                                         None
//                                     }?,
//                                 )
//                                 .is_some()
//                             {
//                                 return None;
//                             }
//                             string.next_if_eq(&',');
//                             string.next_if_eq(&' ');
//                         }
//                         _ => return None,
//                     }
//                 }
//             }
//         }
//     }
// }