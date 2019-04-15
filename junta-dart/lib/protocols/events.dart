import 'errors.dart';

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

abstract class ResEventResult {
  dynamic toJson();

  static ResEventResult fromJson(dynamic json) {
    if (!(json is Map<String, dynamic>)) {
      throw JuntaError(
          "ResEventResult.fromJson expected 'Map<String, dynamic>' got: '${json.runtimeType}'");
    }
    if (json["Ok"] != null) {
      return ResEventOk(json["Ok"]);
    } else if (json["Err"] != null) {
      return ResEventErr(json["Err"]);
    } else {
      throw JuntaError("ResEventResult.fromJson got invalid result");
    }
  }
}

class ResEventOk implements ResEventResult {
  final dynamic value;
  const ResEventOk(this.value);

  dynamic toJson() {
    return {"Ok": value};
  }
}

class ResEventErr implements ResEventResult {
  final dynamic error;
  const ResEventErr(this.error);
  dynamic toJson() {
    return {"Err": error};
  }
}

class ResEventType extends EventType {
  final String name;
  final ResEventResult result;
  const ResEventType(this.name, this.result);

  @override
  toJson() => {
        "Res": [name, result.toJson()]
      };

  static ResEventType fromJson(dynamic json) {
    if (!(json is Map<String, dynamic>)) {
      throw Error();
    }
    if (json["Res"] == null) {
      throw Error();
    }
    var data = json["Res"];

    return ResEventType(data[0] as String, ResEventResult.fromJson(data[1]));
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
