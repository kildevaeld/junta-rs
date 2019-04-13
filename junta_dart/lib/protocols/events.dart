abstract class EventType {
  dynamic toJson();
  const EventType();
}

class ReqEventType extends EventType {
  final String name;
  final dynamic value;
  const ReqEventType(this.name, this.value);

  @override
  toJson() {
    return {
      "Req": [name, value]
    };
  }

  static ReqEventType fromJson(dynamic json) {
    if (!(json is Map<String, dynamic>)) {
      throw Error();
    }
    if (json["Req"] == null) {
      throw Error();
    }
    var data = json["Req"];

    return ReqEventType(data[0] as String, data[1]);
  }
}

class ResEventType extends EventType {
  final String name;
  final dynamic value;
  const ResEventType(this.name, this.value);

  @override
  toJson() => {
        "Res": [name, value]
      };

  static ResEventType fromJson(dynamic json) {
    if (!(json is Map<String, dynamic>)) {
      throw Error();
    }
    if (json["Res"] == null) {
      throw Error();
    }
    var data = json["Res"];

    return ResEventType(data[0] as String, data[1]);
  }
}

class Event {
  final int id;
  final EventType type;

  const Event(this.id, this.type);

  Event copyWith({int id, EventType type}) {
    return Event(id ?? this.id, type ?? this.type);
  }

  dynamic toJson() {
    return {"id": id, "type": type.toJson()};
  }

  static Event fromJson(dynamic json) {
    if (!(json is Map<String, dynamic>)) {
      throw "json not map was ${json.runtimeType}";
    }

    final type = json["type"];
    var t;
    if (type["Res"] != null) {
      t = ResEventType.fromJson(type);
    } else if (type["Req"] != null) {
      t = ReqEventType.fromJson(type);
    }

    return Event(json["id"], t);
  }
}
