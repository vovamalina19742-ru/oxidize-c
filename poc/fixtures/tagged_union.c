#include <stdio.h>
#include <stdbool.h>

typedef enum {
    SHAPE_CIRCLE,
    SHAPE_RECTANGLE
} ShapeType;

typedef struct {
    double radius;
} Circle;

typedef struct {
    double width;
    double height;
} Rectangle;

typedef struct {
    ShapeType type;
    union {
        Circle circle;
        Rectangle rect;
    } data;
} Shape;

double shape_calculate_area(const Shape *s) {
    if (!s) return 0.0;
    switch (s->type) {
        case SHAPE_CIRCLE:
            return 3.141592653589793 * s->data.circle.radius * s->data.circle.radius;
        case SHAPE_RECTANGLE:
            return s->data.rect.width * s->data.rect.height;
        default:
            return 0.0;
    }
}
