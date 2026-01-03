package me.owdding.cats.api;

import java.io.IOException;

public final class CatsException extends IOException {
    public CatsException(String message) {
        super(message);
    }
}
