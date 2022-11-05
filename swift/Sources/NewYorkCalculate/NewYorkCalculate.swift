import NewYorkCalculateRust

public struct NewYorkCalculate {
    public private(set) var text = "Hello, World!"

    public init() {
    }
    
    public static func add(a: Int32, b: Int32) -> Int32 {
        NewYorkCalculateRust.add(a, b)
    }
    
    public static func flip(a: Bool) -> Bool {
        NewYorkCalculateRust.flip(a)
    }
}
