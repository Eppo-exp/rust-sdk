package com.eppo;

class EppoClient {
    // This declares that the static `hello` method will be provided
    // a native library.
    private static native String getStringAssignment(String flagKey, String subjectKey);

    static {
        // This actually loads the shared object that we'll be creating.
        // The actual location of the .so or .dll may differ based on your
        // platform.
        System.loadLibrary("eppo_java");
    }

    // The rest is just regular ol' Java!
    public static void main(String[] args) {
        String output = EppoClient.getStringAssignment("my-flag", "my-subject");
        System.out.println(output);
    }
}
