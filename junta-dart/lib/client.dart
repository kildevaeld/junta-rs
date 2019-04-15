import 'dart:io';
import 'dart:convert';
import 'dart:async';
import './protocols/protocols.dart';
import './context.dart';
import './service.dart';

enum LogLevel { Error, Warn, Info, Debug }

abstract class Logger {
  log(LogLevel level, String msg);
}

class Client extends BaseClient {
  final WebSocket _socket;
  final Service<Context<ClientEvent>, void> service;
  final Logger logger;
  int _seq = 0;
  final Map<int, Completer> _listeners;
  StreamSubscription<dynamic> _subscription;

  Client._internal(this._socket, this._listeners, {this.service, this.logger});

  static Future<Client> connect(String url,
      {IntoService<Context<ClientEvent>, void> service, Logger logger}) async {
    final socket = await WebSocket.connect(url, protocols: ["rust-websocket"]);

    final listeners = Map<int, Completer<dynamic>>();
    var middle = ResponseProtocol(listeners).intoService();
    Service<Context<ClientEvent>, void> resolvedService;
    if (service != null) {
      resolvedService = middle.or(service.intoService());
    } else {
      resolvedService = middle;
    }

    final client = Client._internal(socket, listeners,
        service: resolvedService, logger: logger);
    final ctx = Context(client, ClientConnectEvent());
    if ((await client.service.check(ctx))) await client.service?.call(ctx);
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
    _subscription = _socket.listen(
        (data) async {
          try {
            if (service == null) {
              return;
            }

            final ctx = Context(this, ClientMessageEvent(data));

            if (!(await service.check(ctx))) {
              return;
            }

            await service.call(ctx);
          } catch (e) {
            logger?.log(LogLevel.Error, "could not parse event $e");
          }
        },
        onDone: () {},
        onError: (e) {
          logger?.log(LogLevel.Error, "$e");
        });
  }
}
