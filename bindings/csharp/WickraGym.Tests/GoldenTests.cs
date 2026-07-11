using System.Text.Json;
using System.Text.Json.Nodes;
using Wickra.Gym;
using Xunit;

namespace Wickra.Gym.Tests;

public class GoldenTests
{
    // bindings/csharp/WickraGym.Tests -> repo root -> golden/
    private static string GoldenRoot =>
        Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "..", "golden"));

    /// <summary>
    /// Cross-language golden parity: replay each committed rollout and assert the
    /// response equals the expected JSON. The binding returns the core's canonical
    /// command_json string verbatim, so byte equality is the exact parity check.
    /// Skips cleanly until the golden fixtures land (§4).
    /// </summary>
    [Fact]
    public void GoldenRollouts()
    {
        if (!Directory.Exists(GoldenRoot))
        {
            return; // fixtures not present yet
        }

        foreach (string dir in Directory.EnumerateDirectories(GoldenRoot))
        {
            string specPath = Path.Combine(dir, "spec.json");
            if (!File.Exists(specPath))
            {
                continue;
            }
            string spec = File.ReadAllText(specPath);
            var candles = JsonNode.Parse(File.ReadAllText(Path.Combine(dir, "candles.json")));
            var expected = JsonNode.Parse(File.ReadAllText(Path.Combine(dir, "expected.json")))!.AsObject();

            using var env = new Env(spec);
            env.Command(new JsonObject { ["cmd"] = "load", ["candles"] = candles }.ToJsonString());

            var resetReq = new JsonObject { ["cmd"] = "reset" };
            if (expected.TryGetPropertyValue("seed", out JsonNode? seed) && seed is not null)
            {
                resetReq["seed"] = seed.DeepClone();
            }
            string reset = env.Command(resetReq.ToJsonString());
            AssertJsonEqual(expected["reset"]!, reset);

            JsonArray actions = expected["actions"]!.AsArray();
            JsonArray trajectory = expected["trajectory"]!.AsArray();
            for (int i = 0; i < actions.Count; i++)
            {
                string step = env.Command(
                    new JsonObject { ["cmd"] = "step", ["action"] = actions[i]!.DeepClone() }.ToJsonString());
                AssertJsonEqual(trajectory[i]!, step);
            }
        }
    }

    private static void AssertJsonEqual(JsonNode want, string got)
    {
        string wantCanon = JsonSerializer.Serialize(JsonSerializer.Deserialize<JsonElement>(want.ToJsonString()));
        string gotCanon = JsonSerializer.Serialize(JsonSerializer.Deserialize<JsonElement>(got));
        Assert.Equal(wantCanon, gotCanon);
    }
}
