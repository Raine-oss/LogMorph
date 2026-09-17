// Bridge Test

package io.github.raine.logmorph;

import org.junit.Test;
import java.io.File;
import static org.junit.Assert.*;

// Tests

public class LogMorphBridgeTest {

    @Test
    public void testNativeBridgeCall() {
        File dataDir = new File("target/test-data");
        boolean loaded = LogMorphBridge.loadNative(dataDir);
        assertTrue("LogMorph native engine should load successfully", loaded);

        File fixture = new File("../tests/fixtures/paper_sample.log");
        assertTrue("Fixture must exist", fixture.exists());

        String output = LogMorphBridge.analyzeLog(
            fixture.getAbsolutePath(),
            false,
            null,
            true
        );

        assertNotNull("Output should not be null", output);
        assertTrue("Output should contain execution summary", output.contains("Execution Summary"));
        assertTrue("Output should identify MyCustomPlugin", output.contains("MyCustomPlugin"));
        assertTrue("Output should identify PlayerMoveEvent", output.contains("PlayerMoveEvent"));
    }
}
