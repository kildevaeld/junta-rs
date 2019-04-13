import '../service.dart';

import './types.dart';
import 'dart:async';
import '../context.dart';
import './events.dart';
import './service.dart';

class ResponseProtocol extends Protocol {
  final Map<int, Completer<dynamic>> _listeners;
  ResponseProtocol(this._listeners);

  @override
  Future<void> call(Context<Event> input) async {
    if (input.message.type is ResEventType) {
      final event = input.message.type as ResEventType;
      try {
        await this._listeners[input.message.id].complete(event.value);
        this._listeners.remove(input.message.id);
      } catch (e) {
        this._listeners.remove(input.message.id);
        throw e;
      }
    }

    return null;
  }

  @override
  Future<bool> check(Context<Event> input) async {
    return input.message.type is ResEventType &&
        this._listeners.containsKey(input.message.id);
  }

  @override
  Service<Context<ClientEvent>, void> intoService() {
    return ProtocolService(this);
  }
}
