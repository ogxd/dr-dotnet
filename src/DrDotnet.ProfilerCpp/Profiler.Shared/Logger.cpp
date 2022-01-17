#include "Logger.h"
#include "OS.h"
#include <sstream>
#include <assert.h>
#include <time.h>
#include <iomanip>

Logger& Logger::Get() {
	static Logger logger;
	return logger;
}

void Logger::Shutdown() {
	Get().Term();
}

const char* Logger::LogLevelToString(LogLevel level) {
	switch (level) {
		case LogLevel::Verbose: return "Verbose";
		case LogLevel::Debug: return "Debug";
		case LogLevel::Info: return "Info";
		case LogLevel::Warning: return "Warning";
		case LogLevel::Error: return "Error";
		case LogLevel::Fatal: return "Fatal";
	}
	assert(false);
	return "Unknown";
}

LogLevel Logger::GetLevel() const {
	return _level;
}

void Logger::SetLevel(LogLevel level) {
	_level = level;
}

void Logger::Term() {

}

#ifdef _WINDOWS
#include <Windows.h>
#endif
#include <iostream>

void Logger::DoLog(LogLevel level, const char* text) {

	// build message with time, level, pid, tid, text
	char time[48];
	const auto now = ::time(nullptr);
#ifdef _WINDOWS
	tm lt;
	localtime_s(&lt, &now);
	auto plt = &lt;
#else
	auto plt = localtime(&now);
#endif
	timespec ts;
	timespec_get(&ts, TIME_UTC);
	
	strftime(time, sizeof(time), "%D %T", plt);

	std::cout << text << std::endl;
}

Logger::Logger() {
	auto logDir = OS::ReadEnvironmentVariable("DOTNEXT_LOGDIR");
	if (logDir.empty())
		logDir = OS::GetCurrentDir();

	// build log file path based on current date and time
	auto now = ::time(nullptr);
	char time[64];
#ifdef _WINDOWS
	tm local;
	localtime_s(&local, &now);
	auto tlocal = &local;
#else
	auto tlocal = localtime(&now);
#endif
	::strftime(time, sizeof(time), "DotNext_%F_%H%M%S.log", tlocal);
}
