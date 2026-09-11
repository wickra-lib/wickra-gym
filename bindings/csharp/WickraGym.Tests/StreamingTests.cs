using System.Globalization;
using System.Text;
using System.Text.Json;
using Wickra.Gym;
using Xunit;

namespace WickraGym.Tests;

/// <summary>
/// The streamed rollout equals the batch tensor, through the command boundary.
///
/// The dataset is precomputed once into a fixed feature tensor — that is the
/// batch half — and <c>step()</c> then streams through it as a pure array index.
/// <c>wickra-gym-core</c> proves the two agree in Rust, but that says nothing about the
/// boundary this binding crosses: a binding that mis-serialised an observation
/// row, or truncated it, would hand back numbers that look plausible and are not
/// the tensor's.
///
/// So the rollout is driven twice through this binding and the two must be
/// byte-identical, a longer rollout must agree with a shorter one on the bars
/// they share, and the first observation must be the warmup bar the core
/// computed.
/// </summary>
public class StreamingTests
{
    private const string StreamSpec =
        "{\"dataset_ref\":\"streaming\",\"symbol\":\"TEST\"," +
        "\"observation\":{\"features\":[" +
        "{\"kind\":\"indicator\",\"name\":\"Sma\",\"params\":[3]}," +
        "{\"kind\":\"price\",\"field\":\"close\"}]}," +
        "\"action_space\":{\"type\":\"discrete\",\"n\":3}," +
        "\"reward\":\"pnl\",\"episode\":{\"max_steps\":32,\"warmup\":3}}";

    private const int Steps = 6;

    private static string Candles()
    {
        var sb = new StringBuilder("[");
        for (int i = 0; i < 20; i++)
        {
            if (i > 0)
            {
                sb.Append(',');
            }

            double f = i;
            string N(double v) => v.ToString(CultureInfo.InvariantCulture);
            sb.Append("{\"ts\":").Append(i.ToString(CultureInfo.InvariantCulture))
              .Append(",\"open\":").Append(N(100.0 + f))
              .Append(",\"high\":").Append(N(101.0 + f))
              .Append(",\"low\":").Append(N(99.0 + f))
              .Append(",\"close\":").Append(N(100.0 + f))
              .Append(",\"volume\":1}");
        }

        return sb.Append(']').ToString();
    }

    private static Env Loaded(string spec)
    {
        var env = new Env(spec);
        env.Command("{\"cmd\":\"load\",\"candles\":" + Candles() + "}");
        return env;
    }

    private static string[] Rollout(int steps)
    {
        using var env = Loaded(StreamSpec);
        var trace = new string[steps + 1];
        trace[0] = env.Command("{\"cmd\":\"reset\",\"seed\":7}");
        for (int i = 0; i < steps; i++)
        {
            trace[i + 1] = env.Command("{\"cmd\":\"step\",\"action\":2.0}");
        }

        return trace;
    }

    [Fact]
    public void TheStreamedRolloutIsByteIdenticalWhenReplayed()
    {
        Assert.Equal(Rollout(Steps), Rollout(Steps));
    }

    [Fact]
    public void APrefixOfTheRolloutMatchesAShorterOne()
    {
        string[] longRun = Rollout(Steps);
        string[] shortRun = Rollout(Steps - 2);
        for (int i = 0; i < shortRun.Length; i++)
        {
            Assert.Equal(shortRun[i], longRun[i]);
        }
    }

    [Fact]
    public void TheFirstObservationIsTheWarmupBar()
    {
        using var env = Loaded(StreamSpec);
        string reset = env.Command("{\"cmd\":\"reset\",\"seed\":7}");
        using var doc = JsonDocument.Parse(reset);
        var observation = doc.RootElement.GetProperty("observation");
        // Bar 3 closes at 103; Sma(3) over the three bars ending there is 102.
        Assert.Equal(102.0, observation[0].GetDouble());
        Assert.Equal(103.0, observation[1].GetDouble());
    }

    [Fact]
    public void AWarmupBelowTheIndicatorLookbackIsRefused()
    {
        using var env = new Env(StreamSpec.Replace("\"warmup\":3", "\"warmup\":1"));
        string response = env.Command("{\"cmd\":\"load\",\"candles\":" + Candles() + "}");
        Assert.Contains("warmup", response);
        Assert.DoesNotContain("\"ok\":true", response);
    }
}
