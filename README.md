# Ускорение Rust-кода

Этот проект демонстрирует значительное ускорение простого, но вычислительно сложного Rust-кода.

## Контекст

Первая версия кода была дана в чате стрима и находится по ссылке: [play.rust-lang.org](https://play.rust-lang.org/?version=stable&mode=debug&edition=2015&gist=e09325fe3902b5c770603b3718f7e3d2). Этот код обсуждался на стриме [(ссылка)](https://youtu.be/2E7ERVtHJtc?t=25134). Стример с ником `anton2920` выразил негативное мнение о языке программирования Rust, что автор этого репозитория считает необоснованным. Был проведен рефакторинг кода с целью улучшения его производительности и демонстрации возможностей Rust.

## Оригинальный шейдер

Исходный код шейдера, который был в последствии использован, находится на [Shadertoy BETA](https://www.shadertoy.com/view/WfS3Dd).
Шейдер под названием "Plasma" был создан пользователем [@XorDev](https://x.com/XorDev/status/1894123951401378051?spm=a2ty_o01.29997173.0.0.78645171i270Jj)

GLSL-код шейдера:
```glsl
/*
    "Plasma" by @XorDev
    
    X Post:
    x.com/XorDev/status/1894123951401378051
    
*/                
void mainImage( out vec4 O, in vec2 I )
{
    //Resolution for scaling
    vec2 r = iResolution.xy,
    //Centered, ratio corrected, coordinates
    p = (I+I-r) / r.y,
    //Z depth
    z,
    //Iterator (x=0)
    i,
    //Fluid coordinates
    f = p*(z+=4.-4.*abs(.7-dot(p,p)));
    
    //Clear frag color and loop 8 times
    for(O *= 0.; i.y++<8.;
        //Set color waves and line brightness
        O += (sin(f)+1.).xyyx * abs(f.x-f.y))
        //Add fluid waves
        f += cos(f.yx*i.y+i+iTime)/i.y+.7;
    
    //Tonemap, fade edges and color gradient
    O = tanh(7.*exp(z.x-4.-p.y*vec4(-1,1,2,0))/O);
}
```
Входные данные шейдера (uniforms):
- iResolution: разрешение вьюпорта.
- iTime: время воспроизведения шейдера.
- iChannelTime, iChannelResolution, iMouse, iDate и другие — не используются в этом проекте.

## Улучшения

Реализованы следующие оптимизации:

*   **Параллелизм:** Использование библиотеки `rayon` для распределения вычислений по потокам CPU.
*   **SIMD:** Библиотека `wide`
*   **Релизная сборка:** Использование оптимизаций компилятора Rust.

## Назначение программы. Запуск и сборка

Эта программа рендерит кадр, результаты которого вычисляются на CPU с помощью портированного на Rust шейдера.
Цель — демонстрация возможностей языка Rust в контексте высокопроизводительных вычислений.

Код шейдера был переписан с GLSL на Rust и адаптирован для оптимизации вычислений на CPU.
> Оптимизация не является максимальной, и это не основная цель проекта.
> Важнее показать, как Rust позволяет эффективно использовать SIMD и многопоточность,
> даже при портировании сложных вычислительных задач с GPU.

Рекомендуется запускать с флагами, оптимизирующими код под конкретный процессор:

```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

Сборка соответственно:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

<div align="center">
  <img src="anim.avif" alt="Центрированное изображение" />
</div>