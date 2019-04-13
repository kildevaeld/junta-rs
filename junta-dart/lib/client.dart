import 'dart:io';
import 'dart:convert';
import 'dart:async';
import './protocols/protocols.dart';
import './context.dart';
import './service.dart';

class Client extends BaseClient {
  final WebSocket _socket;
  final Service<Context<ClientEvent>, void> _service;

  int _seq = 0;
  final Map<int, Completer> _listeners;
  StreamSubscription<dynamic> _subscription;

  Client._internal(this._socket, this._listeners, [this._service]);

  static Future<Client> connect(String url,
      [IntoService<Context<ClientEvent>, void> intoService]) async {
    final socket = await WebSocket.connect(url, protocols: ["rust-websocket"]);

    final listeners = Map<int, Completer<dynamic>>();
    var middle = ResponseProtocol(listeners).intoService();
    Service<Context<ClientEvent>, void> service;
    if (intoService != null) {
      service = middle.or(intoService.intoService());
    } else {
      service = middle;
    }

    final client = Client._internal(socket, listeners, service);
    final ctx = Context(client, ClientConnectEvent());
    if ((await client._service.check(ctx))) await client._service?.call(ctx);
    client._listen();
    return client;
  }

  send(dynamic data) {
    _socket.add(data);
  }

  Future<dynamic> request(String method, dynamic args, {bool binary: false}) {
    final event = Event(++this._seq, ReqEventType(method, args));
    final completer = Completer();
    _listeners[event.id] = completer;
    _socket.add(jsonEncode(event));
    return completer.future;
  }

  Future close({int code: 1000, String reason: "NORMAL"}) async {
    await _socket.close(code, reason);

    _listeners.forEach((i, c) {
      c.completeError("close");
    });

    _listeners.clear();
  }

  pause() {
    _subscription?.pause();
  }

  resume() {
    _subscription?.resume();
  }

  _listen() {
    _subscription = _socket.listen((data) async {
      try {
        if (_service == null) {
          return;
        }

        final ctx = Context(this, ClientMessageEvent(data));

        if (!(await _service.check(ctx))) {
          return;
        }

        await _service.call(ctx);
      } catch (e) {
        print("could not parse event $e");
      }
    }, onDone: () {
      print("dont");
    }, onError: (e) {
      print("ERROR $e");
    });
  }
}
