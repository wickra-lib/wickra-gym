package org.wickra.gym;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;

/**
 * A deterministic, Gymnasium-compatible backtest environment over the wickra-gym
 * C ABI (FFM/Panama). Construct from a spec JSON, drive it with command JSON
 * ({@code load}, {@code reset}, {@code step}, {@code spec}, {@code version}) and
 * read back the response JSON — the same protocol as every other binding.
 */
public final class Env implements AutoCloseable {
    private MemorySegment handle;

    /**
     * Construct an environment from an {@code EnvSpec} JSON string.
     *
     * @throws IllegalArgumentException the spec fails to parse or validate
     */
    public Env(String specJson) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment spec = arena.allocateFrom(specJson);
            MemorySegment created = (MemorySegment) Native.NEW.invokeExact(spec);
            if (created.address() == 0) {
                throw new IllegalArgumentException("wickra-gym: invalid spec");
            }
            this.handle = created;
        } catch (RuntimeException | Error e) {
            throw e;
        } catch (Throwable t) {
            throw new RuntimeException(t);
        }
    }

    /**
     * Apply a command JSON and return the response JSON. Starts with a large
     * buffer so a single call suffices; on the rare re-alloc path the core serves
     * the cached response of the same not-yet-delivered command, so a mutating
     * command is never re-run. Domain errors come back in-band as
     * {@code {"ok":false,...}} JSON, not as an exception.
     */
    public String command(String cmdJson) {
        if (handle == null) {
            throw new IllegalStateException("env is closed");
        }
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cmd = arena.allocateFrom(cmdJson);
            long cap = 65536L;
            MemorySegment buf = arena.allocate(cap);
            int n = (int) Native.COMMAND.invokeExact(handle, cmd, buf, cap);
            if (n < 0) {
                throw new IllegalStateException("wickra-gym: command failed (code " + n + ")");
            }
            if (n >= cap) {
                cap = (long) n + 1;
                buf = arena.allocate(cap);
                n = (int) Native.COMMAND.invokeExact(handle, cmd, buf, cap);
                if (n < 0) {
                    throw new IllegalStateException("wickra-gym: command failed (code " + n + ")");
                }
            }
            return buf.getString(0);
        } catch (RuntimeException | Error e) {
            throw e;
        } catch (Throwable t) {
            throw new RuntimeException(t);
        }
    }

    /** The wickra-gym version. */
    public static String version() {
        try {
            MemorySegment ptr = (MemorySegment) Native.VERSION.invokeExact();
            return ptr.reinterpret(Long.MAX_VALUE).getString(0);
        } catch (Throwable t) {
            throw new RuntimeException(t);
        }
    }

    /** Free the native environment handle. */
    @Override
    public void close() {
        if (handle != null) {
            try {
                Native.FREE.invokeExact(handle);
            } catch (Throwable t) {
                throw new RuntimeException(t);
            }
            handle = null;
        }
    }
}
