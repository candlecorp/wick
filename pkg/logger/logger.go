package logger

import (
	"github.com/go-logr/logr"
	"github.com/go-logr/zapr"
	"github.com/mattn/go-colorable"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

var sugar zap.SugaredLogger = *initLogger()
var logger logr.Logger

func Debug(msg string, keysAndValues ...interface{}) {
	sugar.Debugw(msg, keysAndValues...)
}

func Info(msg string, keysAndValues ...interface{}) {
	sugar.Infow(msg, keysAndValues...)
}

func Warn(msg string, keysAndValues ...interface{}) {
	sugar.Warnw(msg, keysAndValues...)
}

func Error(msg string, keysAndValues ...interface{}) {
	sugar.Errorw(msg, keysAndValues...)
}

func initLogger() *zap.SugaredLogger {
	zapLog := initZap(zapcore.InfoLevel)
	logger = zapr.NewLogger(zapLog)
	return zapLog.Sugar()
}

func initZap(logLevel zapcore.Level) *zap.Logger {
	zapConfig := zap.NewDevelopmentEncoderConfig()
	zapConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	zapLog := zap.New(zapcore.NewCore(
		zapcore.NewConsoleEncoder(zapConfig),
		zapcore.AddSync(colorable.NewColorableStdout()),
		logLevel,
	))
	return zapLog
}

func SetLogLevel(logLevel zapcore.Level) {
	zapLog := initZap(logLevel)
	sugar = *zapLog.Sugar()
	logger = zapr.NewLogger(zapLog)
}

func GetLogger() logr.Logger {
	return logger
}
