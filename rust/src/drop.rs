// Help: https://runrust.miraheze.org/wiki/Underscore#Drop_order_for_wildcard_patterns
// https://stackoverflow.com/questions/67000498/what-does-the-equal-actually-do-in-rust-with-without-let

struct MyType {
    a: i32,
}

impl Drop for MyType {
    fn drop(&mut self) {
        println!("MyType is getting dropped: {}", self.a);
    }
}

pub fn test_drop() {
    _ = MyType { a: 5 };
    let _ = MyType { a: 10 };
    let a_ = MyType { a: 20 };
    let _a = MyType { a: 30 };
    println!("Hello from test_drop");
}

/*
MyType is getting dropped: 5
MyType is getting dropped: 10
Hello from test_drop
MyType is getting dropped: 30
MyType is getting dropped: 20
 */
