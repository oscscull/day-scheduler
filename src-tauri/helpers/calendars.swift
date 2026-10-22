// Reads the macOS Calendar app for Quarterdeck, which forbids unsafe Rust and so can't call
// EventKit itself. Prints JSON on stdout; errors go to stderr with a non-zero exit.
//
//   quarterdeck-calendars list
//   quarterdeck-calendars events <name> <from RFC 3339> <to RFC 3339>
//
// <name> is a calendar's title, or "account/title" when two accounts share a title.

import EventKit
import Foundation

struct CalendarInfo: Encodable {
    let title: String
    let account: String
}

struct Event: Encodable {
    let id: String
    let title: String
    let start: Date
    let end: Date
    let allDay: Bool
    let location: String?
    let notes: String?
    let url: String?

    enum CodingKeys: String, CodingKey {
        case id, title, start, end, location, notes, url
        case allDay = "all_day"
    }
}

final class Flag: @unchecked Sendable {
    var value = false
}

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    exit(1)
}

let denied = "Quarterdeck can't read your calendars. Allow it in System Settings → Privacy & Security → Calendars."

/// A store with full access, asking the user the first time.
func openStore() -> EKEventStore {
    switch EKEventStore.authorizationStatus(for: .event) {
    case .fullAccess:
        return EKEventStore()
    case .notDetermined:
        let granted = Flag()
        let done = DispatchSemaphore(value: 0)
        EKEventStore().requestFullAccessToEvents { ok, _ in
            granted.value = ok
            done.signal()
        }
        done.wait()
        guard granted.value else { fail(denied) }
        // A store made before access was granted can stay empty.
        return EKEventStore()
    default:
        fail(denied)
    }
}

func output(_ value: some Encodable) {
    let encoder = JSONEncoder()
    encoder.dateEncodingStrategy = .iso8601
    guard let data = try? encoder.encode(value) else { fail("couldn't encode calendar data") }
    FileHandle.standardOutput.write(data)
}

func calendars(_ store: EKEventStore) -> [EKCalendar] {
    store.calendars(for: .event)
}

func find(_ store: EKEventStore, _ name: String) -> EKCalendar {
    let matches = calendars(store).filter {
        $0.title == name || "\($0.source.title)/\($0.title)" == name
    }
    guard let first = matches.first else {
        fail("there's no calendar called \"\(name)\" in the Calendar app")
    }
    guard matches.count == 1 else {
        fail("more than one calendar is called \"\(name)\"; name it as \"account/\(name)\"")
    }
    return first
}

func date(_ text: String) -> Date {
    guard let date = ISO8601DateFormatter().date(from: text) else {
        fail("'\(text)' is not an RFC 3339 time")
    }
    return date
}

func event(_ e: EKEvent) -> Event {
    Event(
        // The iCal UID plus the start, so each occurrence of a repeating event is distinct.
        id: "\(e.calendarItemExternalIdentifier ?? e.calendarItemIdentifier)@\(ISO8601DateFormatter().string(from: e.startDate))",
        title: e.title ?? "",
        start: e.startDate,
        end: e.endDate,
        allDay: e.isAllDay,
        location: e.location,
        notes: e.notes,
        url: e.url?.absoluteString
    )
}

let args = CommandLine.arguments.dropFirst()
switch args.first {
case "list":
    let store = openStore()
    output(calendars(store).map { CalendarInfo(title: $0.title, account: $0.source.title) })
case "events" where args.count == 4:
    let rest = Array(args.dropFirst())
    let store = openStore()
    let calendar = find(store, rest[0])
    let predicate = store.predicateForEvents(
        withStart: date(rest[1]), end: date(rest[2]), calendars: [calendar])
    output(store.events(matching: predicate).filter { $0.status != .canceled }.map(event))
default:
    fail("usage: quarterdeck-calendars list | events <name> <from> <to>")
}
