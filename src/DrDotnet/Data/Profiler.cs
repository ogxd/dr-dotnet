using Microsoft.Diagnostics.NETCore.Client;
using System;
using System.IO;
using System.Text;

namespace DrDotnet
{
    public class Profiler
    {
        public Guid ProfilerId { get; set; }

        public string Name { get; set; }

        public string Description { get; set; }

        public Guid StartProfilingSession(int processId, ILogger logger)
        {
            string strExeFilePath = System.Reflection.Assembly.GetExecutingAssembly().Location;
            string strWorkPath = Path.GetDirectoryName(strExeFilePath);
            string profilerDll = Path.Combine(strWorkPath, "Profiler.Windows.dll");

            var sessionId = Guid.NewGuid();

            DiagnosticsClient client = new DiagnosticsClient(processId);
            client.AttachProfiler(TimeSpan.FromSeconds(10), new Guid("{805A308B-061C-47F3-9B30-F785C3186E84}"), profilerDll/*, Encoding.UTF8.GetBytes(sessionId.ToString() + "\0")*/);

            logger.Log($"Attached profiler {ProfilerId} with session {sessionId} to process {processId}");

            return sessionId;
        }
    }
}