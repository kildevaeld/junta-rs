abstract class IntoService<I, O> {
  Service<I, O> intoService();
}

abstract class Service<I, O> implements IntoService<I, O> {
  Future<O> call(I input);

  Future<bool> check(I input) async => true;

  @override
  Service<I, O> intoService() {
    return this;
  }

  Service<I, O> or(Service<I, O> service) {
    return ServiceChain(this, service);
  }
}

class ServiceChain<I, O> extends Service<I, O> {
  final Service<I, O> s1;
  final Service<I, O> s2;

  ServiceChain(this.s1, this.s2);

  @override
  Future<O> call(I input) async {
    if (await s1.check(input)) {
      return s1.call(input);
    } else {
      return s2.call(input);
    }
  }

  @override
  Future<bool> check(I input) async {
    return (await s1.check(input)) || (await s2.check(input));
  }
}
