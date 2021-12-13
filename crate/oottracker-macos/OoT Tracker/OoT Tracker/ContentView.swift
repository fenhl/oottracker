import SwiftUI

struct ContentView: View {
    var body: some View {
        VStack {
            Text("OoT Tracker \(versionWrapper())")
                .padding()
            Image("eyeball_frog")
                .onTapGesture {
                    print("ribbit")
                }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
