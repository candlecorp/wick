package logger

import (
	"github.com/go-logr/logr"
	"github.com/go-logr/zapr"
	"github.com/mattn/go-colorable"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

func GetLogger(logLevel zapcore.Level) logr.Logger {
	// Initialize logger
	zapConfig := zap.NewDevelopmentEncoderConfig()
	zapConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	zapLog := zap.New(zapcore.NewCore(
		zapcore.NewConsoleEncoder(zapConfig),
		zapcore.AddSync(colorable.NewColorableStdout()),
		logLevel,
	))
	//zapLog, err := zapConfig.Build()
	//zapLog, err := zap.NewProduction()
	// if err != nil {
	//      panic(err)
	// }
	// zapLog := zap.NewExample()
	log := zapr.NewLogger(zapLog)
	return log
}
