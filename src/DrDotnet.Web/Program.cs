using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Hosting;

namespace DrDotnet.Web;

public class Program
{
    public static void Main(string[] args)
    {
        CreateHostBuilder(args).Build().Run();
    }

    public static IHostBuilder CreateHostBuilder(string[] args) =>
        Host.CreateDefaultBuilder(args)
            .ConfigureWebHostDefaults(webBuilder =>
            {
                webBuilder.UseStartup<Startup>();
                //webBuilder.UseSetting("https_port", "51376");
                //webBuilder.UseUrls("http://localhost:51376");
                webBuilder.UseUrls(@"http://*:92");
                webBuilder.UseSetting(WebHostDefaults.DetailedErrorsKey, "true");
            });
}