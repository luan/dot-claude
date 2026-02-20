import AppKit
// Usage: gen-circle <#hexcolor> <output.png> [symbol]
// Symbols: check, lock, chat (or omit for plain circle)
// Uses SF Symbols for clean monochrome glyphs.
guard CommandLine.arguments.count >= 3 else { exit(1) }
let hex = CommandLine.arguments[1].trimmingCharacters(in: CharacterSet(charactersIn: "#"))
let symbol = CommandLine.arguments.count >= 4 ? CommandLine.arguments[3] : ""
let r = CGFloat(UInt8(hex.prefix(2), radix: 16)!) / 255
let g = CGFloat(UInt8(hex.dropFirst(2).prefix(2), radix: 16)!) / 255
let b = CGFloat(UInt8(hex.suffix(2), radix: 16)!) / 255
let size = NSSize(width: 128, height: 128)
let img = NSImage(size: size)
img.lockFocus()
NSColor(red: r, green: g, blue: b, alpha: 1).setFill()
NSBezierPath(ovalIn: NSRect(x: 8, y: 8, width: 112, height: 112)).fill()

if !symbol.isEmpty {
    let sfName: String
    switch symbol {
    case "check": sfName = "checkmark"
    case "lock": sfName = "lock.fill"
    case "chat": sfName = "bubble.left.fill"
    default: sfName = symbol
    }
    if let sfImage = NSImage(systemSymbolName: sfName, accessibilityDescription: nil) {
        let config = NSImage.SymbolConfiguration(pointSize: 48, weight: .bold)
        let configured = sfImage.withSymbolConfiguration(config)!
        let tinted = NSImage(size: configured.size, flipped: false) { rect in
            NSColor(white: 0, alpha: 0.5).set()
            configured.draw(in: rect)
            return true
        }
        let symSize = tinted.size
        let origin = NSPoint(x: (size.width - symSize.width) / 2, y: (size.height - symSize.height) / 2)
        tinted.draw(at: origin, from: .zero, operation: .sourceOver, fraction: 1)
    }
}

img.unlockFocus()
guard let tiff = img.tiffRepresentation,
      let bmp = NSBitmapImageRep(data: tiff),
      let png = bmp.representation(using: .png, properties: [:]) else { exit(1) }
try! png.write(to: URL(fileURLWithPath: CommandLine.arguments[2]))
