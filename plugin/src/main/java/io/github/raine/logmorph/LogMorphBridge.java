// Bridge

package io.github.raine.logmorph;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

// Native Loader

public class LogMorphBridge {
    private static boolean loaded = false;

    public static synchronized boolean loadNative(File dataFolder) {
        if (loaded) {
            return true;
        }

        try {
            String os = System.getProperty("os.name").toLowerCase();
            String arch = System.getProperty("os.arch").toLowerCase();

            String resourcePath;
            String libName;

            if (os.contains("linux")) {
                libName = "liblogmorph.so";
                if (arch.contains("aarch64") || arch.contains("arm64")) {
                    resourcePath = "/natives/linux-aarch64/liblogmorph.so";
                } else {
                    resourcePath = "/natives/linux-x86_64/liblogmorph.so";
                }
            } else if (os.contains("win")) {
                libName = "logmorph.dll";
                resourcePath = "/natives/windows-x86_64/logmorph.dll";
            } else if (os.contains("mac")) {
                libName = "liblogmorph.dylib";
                resourcePath = "/natives/macos/liblogmorph.dylib";
            } else {
                libName = "liblogmorph.so";
                if (arch.contains("aarch64") || arch.contains("arm64")) {
                    resourcePath = "/natives/linux-aarch64/liblogmorph.so";
                } else {
                    resourcePath = "/natives/linux-x86_64/liblogmorph.so";
                }
            }

            File nativesDir = new File(dataFolder, "natives");
            if (!nativesDir.exists()) {
                nativesDir.mkdirs();
            }

            File extractedLib = new File(nativesDir, libName);

            try (InputStream in = LogMorphBridge.class.getResourceAsStream(resourcePath)) {
                if (in != null) {
                    try (OutputStream out = new FileOutputStream(extractedLib)) {
                        byte[] buf = new byte[8192];
                        int len;
                        while ((len = in.read(buf)) > 0) {
                            out.write(buf, 0, len);
                        }
                    }
                }
            }

            if (extractedLib.exists()) {
                System.load(extractedLib.getAbsolutePath());
                loaded = true;
                return true;
            } else {
                System.loadLibrary("logmorph");
                loaded = true;
                return true;
            }
        } catch (Throwable t) {
            t.printStackTrace();
            return false;
        }
    }

    public static boolean isLoaded() {
        return loaded;
    }

    // Native Declaration

    public static native String analyzeLog(
        String logPath,
        boolean summaryOnly,
        String pluginFilter,
        boolean noColor
    );

    public static native String exportJson(String logPath);
}
