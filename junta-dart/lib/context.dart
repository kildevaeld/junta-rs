abstract class BaseClient {
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
}
