import '../context.dart';
import '../service.dart';
import './protocols.dart';
import 'dart:convert';

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
      throw "something";
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
      return false;
    } catch (e) {
      //print("invalid json $e");
      return false;
    }
  }
}
