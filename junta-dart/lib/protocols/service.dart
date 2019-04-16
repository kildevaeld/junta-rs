import '../context.dart';
import '../service.dart';
import './protocols.dart';
import 'dart:convert';
import 'errors.dart';

class ProtocolService extends Service<Context<ClientEvent>, void> {
  final Protocol protocol;

  ProtocolService(this.protocol);

  @override
  Future call(Context<ClientEvent> input) async {
    if (input.message is ClientMessageEvent) {
      final msg = input.message as ClientMessageEvent;
      final event = Event.fromJson(jsonDecode(msg.data));

      return await protocol.call(input.copyWith(event));
    } else {
      throw JuntaError("input.message is not a ClientMessageEvent");
    }
  }

  @override
  Future<bool> check(Context<ClientEvent> input) async {
    try {
      if (input.message is ClientMessageEvent) {
        final msg = input.message as ClientMessageEvent;

        final event = Event.fromJson(jsonDecode(msg.data));

        return await protocol.check(input.copyWith(event));
      }
      input.logger?.debug(
          "input was '${input.message.runtimeType}'. Expected: ClientMessageEvent");
      return false;
    } catch (e) {
      input.logger?.debug("parse error $e");
      return false;
    }
  }
}
