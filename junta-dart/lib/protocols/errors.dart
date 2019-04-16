class JuntaError implements Exception {
  final String message;
  const JuntaError([this.message]);

  @override
  String toString() {
    // TODO: implement toString
    return message;
  }
}

class JuntaRequestError extends JuntaError {
  final int code;
  const JuntaRequestError(this.code, String message) : super(message);

  dynamic toJson() {
    return {"code": code, "message": message};
  }

  static JuntaRequestError fromJson(dynamic json) {
    if (!(json is Map<String, dynamic>)) {
      throw JuntaError(
          "JuntaRequestError.fromJson expected 'Map<String, dynamic>' got: '${json.runtimeType}'");
    }
    return JuntaRequestError(json["code"] as int, json["message"]);
  }

  @override
  String toString() {
    // TODO: implement toString
    return "JuntaRequestError($code, $message)";
  }
}
