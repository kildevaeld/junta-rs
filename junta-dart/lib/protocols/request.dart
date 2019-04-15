import './types.dart';
import '../context.dart';
import './events.dart';
import 'dart:convert';

abstract class RequestProtocolHandler {
  Future<dynamic> call(Context<dynamic> input);
  Future<bool> check(Context<dynamic> input);
}

class RequestProtocol extends Protocol {
  final String name;
  final RequestProtocolHandler handler;

  RequestProtocol(this.name, this.handler);

  Future<void> call(Context<Event> input) {
    final type = input.message.type;
    if (!(type is ReqEventType)) {
      throw "not a req event type";
    }

    final name = (type as ReqEventType).name;
    return handler(input.copyWith((type as ReqEventType).value)).then((value) {
      final event =
          input.message.copyWith(type: ResEventType(name, ResEventOk(value)));
      input.client.send(jsonEncode(event));
    }, onError: (err) {
      final event = input.message
          .copyWith(type: ResEventType(name, ResEventErr(err.toString())));
      input.client.send(jsonEncode(event));
    });
  }

  Future<bool> check(Context<Event> input) async {
    final type = input.message.type;
    return type is ReqEventType && type.name == this.name;
  }

  // @override
  // Service<Context<ClientEvent>, void> intoService() {
  //   return ProtocolService(this);
  // }
}

typedef RequestProtocolFunc = Future<dynamic> Function(Context<dynamic> input);

class RequestProtocolFn extends RequestProtocolHandler {
  RequestProtocolFunc fn;

  RequestProtocolFn(this.fn);

  Future<void> call(Context<dynamic> input) {
    return fn(input);
  }

  Future<bool> check(Context<dynamic> input) async {
    return true;
  }

  RequestProtocol intoProtocol(String name) => RequestProtocol(name, this);
}
