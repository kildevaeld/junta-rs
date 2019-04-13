import '../context.dart';
import '../service.dart';
import './events.dart';
import './service.dart';

abstract class IntoProtocol {
  Protocol intoProtocol();
}

abstract class Protocol
    implements IntoProtocol, IntoService<Context<ClientEvent>, void> {
  Future<void> call(Context<Event> input);
  Future<bool> check(Context<Event> input);

  @override
  Protocol intoProtocol() {
    return this;
  }

  @override
  Service<Context<ClientEvent>, void> intoService() {
    return ProtocolService(this);
  }
}
