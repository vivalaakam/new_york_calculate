import XCTest
@testable import NewYorkCalculate

final class NewYorkCalculateTests: XCTestCase {
    func testExample() throws {
        // This is an example of a functional test case.
        // Use XCTAssert and related functions to verify your tests produce the correct
        // results.
        XCTAssertEqual(NewYorkCalculate().text, "Hello, World!")
    }
    
    func testAdd() throws {
        let actual = NewYorkCalculate.add(a: 1, b: 2)
        
        XCTAssertEqual(3, actual)
    }
}
