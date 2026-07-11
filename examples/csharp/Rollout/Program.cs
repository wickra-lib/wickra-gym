// A runnable C# example: load the momentum_discrete spec and its candle dataset
// over the wickra-gym C ABI binding, then drive a fixed long policy and print
// each step. The rollout is byte-identical to the other language examples on the
// same seed.
//
//   cargo build --release -p wickra-gym-c
//   dotnet run --project examples/csharp/Rollout

using System.Text.Json;
using Wickra.Gym;

static string DataDir()
{
    string dir = AppContext.BaseDirectory;
    for (int i = 0; i < 10; i++)
    {
        string candidate = Path.Combine(dir, "examples", "data");
        if (Directory.Exists(candidate))
        {
            return candidate;
        }
        dir = Path.GetDirectoryName(dir.TrimEnd(Path.DirectorySeparatorChar))!;
    }
    throw new DirectoryNotFoundException("examples/data not found");
}

string data = DataDir();
string spec = File.ReadAllText(Path.Combine(data, "specs", "momentum_discrete.json"));
string candles = File.ReadAllText(Path.Combine(data, "candles.json"));

using var env = new Env(spec);
env.Command($"{{\"cmd\":\"load\",\"candles\":{candles}}}");
Console.WriteLine($"reset: {env.Command("{\"cmd\":\"reset\",\"seed\":42}")}");

double equity = 0.0;
for (int step = 0; ; step++)
{
    using JsonDocument doc = JsonDocument.Parse(env.Command("{\"cmd\":\"step\",\"action\":2}"));
    JsonElement root = doc.RootElement;
    double reward = root.GetProperty("reward").GetDouble();
    bool terminated = root.GetProperty("terminated").GetBoolean();
    bool truncated = root.GetProperty("truncated").GetBoolean();
    equity += reward;
    Console.WriteLine(
        $"step {step}: reward {reward:+0.000000;-0.000000}  equity {equity:+0.000000;-0.000000}  " +
        $"terminated={terminated} truncated={truncated}");
    if (terminated || truncated)
    {
        break;
    }
}
