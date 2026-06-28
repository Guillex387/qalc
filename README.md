# TFG: Qalc

## Tabla de Contenidos

- [1. Introducción y Motivación](#1-introducción-y-motivación)
- [2. Estado del Arte y Tecnologías Actuales](#2-estado-del-arte-y-tecnologías-actuales)
- [3. Diseño del Lenguaje y Especificación](#3-diseño-del-lenguaje-y-especificación)
- [4. Arquitectura del Compilador](#4-arquitectura-del-compilador)
- [5. Análisis Semántico y Sistema de Tipos](#5-análisis-semántico-y-sistema-de-tipos)
- [6. Generación de Código y Backend](#6-generación-de-código-y-backend)
- [7. Metodología de Desarrollo y Pruebas](#7-metodología-de-desarrollo-y-pruebas)
- [8. Conclusiones y Trabajo Futuro](#8-conclusiones-y-trabajo-futuro)

Este documento prepara la redacción de la memoria técnica del proyecto. Aquí se estructura cada capítulo con su contenido principal y las ideas clave que deben desarrollarse.

## 1. Introducción y Motivación

Contenido:
- Define qué es la computación cuántica.
- Explica por qué los lenguajes actuales basados en circuitos son una barrera para desarrolladores de software.
- Muestra el problema de la complejidad física y la falta de abstracción en los SDK de bajo nivel.

El Gancho:
- Presenta la propuesta del lenguaje: abstraer la complejidad física para permitir algoritmos más complejos.
- Menciona conceptos clave como la gestión de la "uncomputation".
- Resalta la importancia de un sistema de tipos fuerte y seguro para programación cuántica.

## 2. Estado del Arte y Tecnologías Actuales

Contenido:
- Comparativa detallada entre SDKs de bajo nivel (Qiskit, Cirq, OpenQASM) y lenguajes de alto nivel (Silq, Q#).
- Explica las fortalezas y debilidades de cada alternativa.

Punto clave:
- Analiza las carencias actuales: verbosidad, falta de sistemas de tipos cuánticos fuertes, gestión manual de recursos cuánticos.
- Justifica por qué se decide diseñar un lenguaje propio con un modelo semántico mejor adaptado.

## 3. Diseño del Lenguaje y Especificación

Contenido:
- Describe la visión del lenguaje y su filosofía de diseño.
- Define la sintaxis general, las palabras clave principales y el sistema de tipos propuesto.
- Incluye la gramática en formato EBNF.
- Añade ejemplos de código ideal para mostrar cómo se expresaría un algoritmo cuántico.

Importante:
- Aunque el compilador no esté terminado al 100%, este capítulo debe estar completo como plano de diseño.
- Debe mostrar claramente la intención del lenguaje y sus reglas básicas.

## 4. Arquitectura del Compilador

Contenido:
- Detalla la infraestructura técnica ya montada.
- Explica el pipeline de compilación y la abstracción `CompilerStep`.
- Muestra cómo ese diseño permite modularidad y extensibilidad.

Pipeline de Compilación:
- Describe la secuencia de fases: parser, normalizador, typechecker, backend.
- Explica cómo cada fase transforma el AST y qué ganamos con ello.

Análisis Léxico y Sintáctico:
- Describe el parser y el AST modular.
- Resalta la gestión de metadatos dinámicos para rastreo de nodos y diagnóstico.

Sistema de Diagnósticos:
- Explica cómo el compilador comunica errores al usuario.
- Incluye detalles de mensajes claros, punteros a línea/columna y visualización de spans.

## 5. Análisis Semántico y Sistema de Tipos

Contenido:
- Describe cómo funciona el typechecker propuesto.
- Menciona la tabla de símbolos y la gestión de ámbitos (scopes).
- Explica las reglas de tipado cuántico planificadas.

Ejemplos de reglas:
- Evitar la clonación de cúbits.
- Gestión de recursos lineales y restricciones de uso.
- Verificación de operaciones válidas sobre cúbits y tipos clásicos.

Si alguna regla no está implementada, debe describirse como parte del diseño lógico.

## 6. Generación de Código y Backend

Contenido:
- Describe el proceso de transformación del AST a un lenguaje de salida, por ejemplo OpenQASM.
- Explica la estrategia y el patrón de diseño usado para recorrer el árbol y emitir instrucciones.

Estrategia:
- Si se usa un patrón como Visitor, descríbelo.
- Si el backend está parcial, preséntalo como un módulo de validación básica de circuitos.

## 7. Metodología de Desarrollo y Pruebas

Contenido:
- Describe las herramientas usadas (Rust, posibles scripts de soporte, etc.).
- Explica la estructura del desarrollo del proyecto.

Validación:
- Añade casos de prueba ya procesables por el compilador.
- Muestra el código fuente de entrada y el AST generado para ejemplos concretos.

## 8. Conclusiones y Trabajo Futuro

Contenido:
- Resume los logros actuales: arquitectura modular, parser robusto, sistema de diagnóstico bien definido.
- Convierte lo que falta en una ventaja: hoja de ruta para librería estándar y optimizaciones avanzadas.
- Muestra que el proyecto es escalable y profesional, con una base sólida para continuar.
