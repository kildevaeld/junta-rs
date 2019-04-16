enum LogLevel { Error, Warn, Info, Debug }

abstract class Logger {
  log(LogLevel level, String msg);
  debug(String msg) {
    log(LogLevel.Debug, msg);
  }

  error(String msg) {
    log(LogLevel.Error, msg);
  }
}

class StdioLogger extends Logger {
  @override
  log(LogLevel level, String msg) {
    print("[${this._levelToString(level)}]: $msg");
  }

  _levelToString(LogLevel level) {
    switch (level) {
      case LogLevel.Debug:
        return "DEBUG";
      case LogLevel.Error:
        return "ERROR";
      case LogLevel.Info:
        return "INFO";
      case LogLevel.Warn:
        return "WARN";
    }
  }
}
