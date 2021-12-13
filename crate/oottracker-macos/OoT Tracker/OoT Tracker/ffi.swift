func versionWrapper() -> String {
    let result = version_string()
    let swift_result = String(cString: result!)
    string_free(UnsafeMutablePointer(mutating: result))
    return swift_result
}
