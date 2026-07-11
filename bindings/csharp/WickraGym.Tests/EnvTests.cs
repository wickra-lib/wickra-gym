using System.Text.Json;
using Wickra.Gym;
using Xunit;

namespace Wickra.Gym.Tests;

public class EnvTests
{
    private const string Spec = """
        {"dataset_ref":"smoke","symbol":"TEST",
         "observation":{"features":[{"kind":"price","field":"close"}]},
         "action_space":{"type":"discrete","n":3},
         "reward":"pnl","episode":{"max_steps":100,"warmup":0}}
        """;

    private static string LoadCommand()
    {
        var candles = Enumerable.Range(0, 5).Select(i => new
        {
            ts = i,
            open = 100.0 + i,
            high = 100.0 + i,
            low = 100.0 + i,
            close = 100.0 + i,
        });
        return JsonSerializer.Serialize(new { cmd = "load", candles });
    }

    [Fact]
    public void Version_IsNonEmpty()
    {
        Assert.False(string.IsNullOrEmpty(Env.Version()));
    }

    [Fact]
    public void Load_Reset_Step()
    {
        using var env = new Env(Spec);
        Assert.Equal("""{"ok":true}""", env.Command(LoadCommand()));

        string reset = env.Command("""{"cmd":"reset"}""");
        Assert.Contains("\"observation\"", reset);

        string step = env.Command("""{"cmd":"step","action":2}""");
        using var doc = JsonDocument.Parse(step);
        Assert.Equal(1.0, doc.RootElement.GetProperty("reward").GetDouble());
        Assert.False(doc.RootElement.GetProperty("terminated").GetBoolean());
    }

    [Fact]
    public void BadSpec_Throws()
    {
        Assert.Throws<ArgumentException>(() => new Env("""{"not":"a spec"}"""));
    }
}
