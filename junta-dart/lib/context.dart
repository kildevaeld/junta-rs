import 'logger.dart';

abstract class BaseClient {
  final Logger logger;
  const BaseClient(this.logger);
  send(dynamic data);
}

abstract class ClientEvent {}

class ClientConnectEvent extends ClientEvent {}

class ClientMessageEvent extends ClientEvent {
  final String data;
  ClientMessageEvent(this.data);
}

class Context<E> {
  final BaseClient client;
  final E message;

  Context(this.client, this.message);

  Context<T> copyWith<T>(T input) {
    return Context(this.client, input);
  }

  Logger get logger => client.logger;
}
